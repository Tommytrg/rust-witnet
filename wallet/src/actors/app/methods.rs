use actix::utils::TimerFunc;
use futures::future;

use super::*;
use crate::actors::*;
use crate::{model, repository, types::Hashable as _};
use witnet_data_structures::chain::InventoryItem;

impl App {
    pub fn start(params: Params) -> Addr<Self> {
        let actor = Self {
            params,
            state: Default::default(),
        };

        actor.start()
    }

    /// Return a new subscription id for a session.
    pub fn next_subscription_id(
        &mut self,
        session_id: types::SessionId,
    ) -> Result<types::SubscriptionId> {
        if self.state.is_session_active(&session_id) {
            // We are re-using the session id as the subscription id, this is because using a number
            // can let any client call the unsubscribe method for any other session.
            Ok(types::SubscriptionId::String(session_id.into()))
        } else {
            Err(Error::SessionNotFound)
        }
    }

    /// Try to create a subscription and store it in the session. After subscribing, events related
    /// to wallets unlocked by this session will be sent to the client.
    pub fn subscribe(
        &mut self,
        session_id: types::SessionId,
        _subscription_id: types::SubscriptionId,
        sink: types::Sink,
    ) -> Result<()> {
        self.state.subscribe(&session_id, sink)
    }

    /// Remove a subscription.
    pub fn unsubscribe(&mut self, id: &types::SubscriptionId) -> Result<()> {
        // Session id and subscription id are currently the same thing. See comment in
        // next_subscription_id method.
        self.state.unsubscribe(id)
    }

    /// Generate a receive address for the wallet's current account.
    pub fn generate_address(
        &mut self,
        session_id: types::SessionId,
        wallet_id: String,
        label: Option<String>,
    ) -> ResponseActFuture<model::Address> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            move |wallet, slf: &mut Self, _| {
                slf.params
                    .worker
                    .send(worker::GenAddress(wallet, label))
                    .flatten()
                    .map_err(From::from)
                    .into_actor(slf)
            },
        );

        Box::new(f)
    }

    /// Get a list of addresses generated by a wallet.
    pub fn get_addresses(
        &mut self,
        session_id: types::SessionId,
        wallet_id: String,
        offset: u32,
        limit: u32,
    ) -> ResponseActFuture<model::Addresses> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            move |wallet, slf: &mut Self, _| {
                slf.params
                    .worker
                    .send(worker::GetAddresses(wallet, offset, limit))
                    .flatten()
                    .map_err(From::from)
                    .into_actor(slf)
            },
        );

        Box::new(f)
    }

    /// Get a list of addresses generated by a wallet.
    pub fn get_balance(
        &mut self,
        session_id: types::SessionId,
        wallet_id: String,
    ) -> ResponseActFuture<model::Balance> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            move |wallet, slf: &mut Self, _| {
                slf.params
                    .worker
                    .send(worker::GetBalance(wallet))
                    .flatten()
                    .map_err(From::from)
                    .into_actor(slf)
            },
        );

        Box::new(f)
    }

    /// Get a list of transactions associated to a wallet account.
    pub fn get_transactions(
        &mut self,
        session_id: types::SessionId,
        wallet_id: String,
        offset: u32,
        limit: u32,
    ) -> ResponseActFuture<model::Transactions> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            move |wallet, slf: &mut Self, _| {
                slf.params
                    .worker
                    .send(worker::GetTransactions(wallet, offset, limit))
                    .flatten()
                    .map_err(From::from)
                    .into_actor(slf)
            },
        );

        Box::new(f)
    }

    /// Run a RADRequest and return the computed result.
    pub fn run_rad_request(&self, req: types::RADRequest) -> ResponseFuture<types::RadonTypes> {
        let f = self
            .params
            .worker
            .send(worker::RunRadRequest(req))
            .flatten()
            .map_err(From::from);

        Box::new(f)
    }

    /// Generate a random BIP39 mnemonics sentence
    pub fn generate_mnemonics(&self, length: types::MnemonicLength) -> ResponseFuture<String> {
        let f = self
            .params
            .worker
            .send(worker::GenMnemonic(length))
            .map_err(From::from);

        Box::new(f)
    }

    /// Forward a Json-RPC call to the node.
    pub fn forward(
        &mut self,
        method: String,
        params: types::RpcParams,
    ) -> ResponseFuture<types::Json> {
        match &self.params.client {
            Some(addr) => {
                let req = types::RpcRequest::method(method)
                    .timeout(self.params.requests_timeout)
                    .params(params)
                    .expect("params failed serialization");
                let f = addr.send(req).flatten().map_err(From::from);

                Box::new(f)
            }
            None => {
                let f = future::err(Error::NodeNotConnected);

                Box::new(f)
            }
        }
    }

    /// Get public info of all the wallets stored in the database.
    pub fn wallet_infos(&self) -> ResponseFuture<Vec<model::Wallet>> {
        let f = self
            .params
            .worker
            .send(worker::WalletInfos)
            .flatten()
            .map_err(From::from);

        Box::new(f)
    }

    /// Create an empty HD Wallet.
    pub fn create_wallet(
        &self,
        password: types::Password,
        seed_source: types::SeedSource,
        name: Option<String>,
        caption: Option<String>,
    ) -> ResponseFuture<String> {
        let f = self
            .params
            .worker
            .send(worker::CreateWallet(name, caption, password, seed_source))
            .flatten()
            .map_err(From::from);

        Box::new(f)
    }

    /// Lock a wallet, that is, remove its encryption/decryption key from the list of known keys and
    /// close the session.
    ///
    /// This means the state of this wallet won't be updated with information received from the
    /// node.
    pub fn lock_wallet(&mut self, session_id: types::SessionId, wallet_id: String) -> Result<()> {
        self.state.remove_wallet(&session_id, &wallet_id)
    }

    /// Load a wallet's private information and keys in memory.
    pub fn unlock_wallet(
        &self,
        wallet_id: String,
        password: types::Password,
    ) -> ResponseActFuture<types::UnlockedWallet> {
        let f = self
            .params
            .worker
            .send(worker::UnlockWallet(wallet_id.clone(), password))
            .flatten()
            .map_err(|err| match err {
                worker::Error::WalletNotFound => {
                    validation_error(field_error("wallet_id", "Wallet not found"))
                }
                worker::Error::WrongPassword => {
                    validation_error(field_error("password", "Wrong password"))
                }
                err => From::from(err),
            })
            .into_actor(self)
            .and_then(move |res, slf: &mut Self, _| {
                let types::UnlockedSessionWallet {
                    wallet,
                    session_id,
                    data,
                } = res;

                slf.state
                    .create_session(session_id.clone(), wallet_id, Arc::new(wallet));

                fut::ok(types::UnlockedWallet { data, session_id })
            });

        Box::new(f)
    }

    pub fn create_vtt(
        &self,
        session_id: &types::SessionId,
        wallet_id: &str,
        vtt_params: types::VttParams,
    ) -> ResponseActFuture<types::Transaction> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            move |wallet, slf: &mut Self, _| {
                slf.params
                    .worker
                    .send(worker::CreateVtt(wallet, vtt_params))
                    .flatten()
                    .map_err(|err| match err {
                        worker::Error::Repository(repository::Error::InsufficientBalance) => {
                            validation_error(field_error(
                                "balance",
                                "Wallet account has not enough balance",
                            ))
                        }
                        err => From::from(err),
                    })
                    .into_actor(slf)
            },
        );

        Box::new(f)
    }

    pub fn create_data_req(
        &self,
        session_id: &types::SessionId,
        wallet_id: &str,
        params: types::DataReqParams,
    ) -> ResponseActFuture<types::Transaction> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            move |wallet, slf: &mut Self, _| {
                slf.params
                    .worker
                    .send(worker::CreateDataReq(wallet, params))
                    .flatten()
                    .map_err(|err| match err {
                        worker::Error::Repository(repository::Error::InsufficientBalance) => {
                            validation_error(field_error(
                                "balance",
                                "Wallet account has not enough balance",
                            ))
                        }
                        err => From::from(err),
                    })
                    .into_actor(slf)
            },
        );

        Box::new(f)
    }

    /// Perform all the tasks needed to properly stop the application.
    pub fn stop(&self) -> ResponseFuture<()> {
        let fut = self
            .params
            .worker
            .send(worker::FlushDb)
            .map_err(internal_error)
            .and_then(|result| result.map_err(internal_error));

        Box::new(fut)
    }

    /// Return a timer function that can be scheduled to expire the session after the configured time.
    pub fn set_session_to_expire(&self, session_id: types::SessionId) -> TimerFunc<Self> {
        log::debug!(
            "Session {} will expire in {} seconds.",
            &session_id,
            self.params.session_expires_in.as_secs()
        );

        TimerFunc::new(
            self.params.session_expires_in,
            move |slf: &mut Self, _ctx| match slf.close_session(session_id.clone()) {
                Ok(_) => log::info!("Session {} closed", session_id),
                Err(err) => log::error!("Session {} couldn't be closed: {}", session_id, err),
            },
        )
    }

    /// Remove a session from the list of active sessions.
    pub fn close_session(&mut self, session_id: types::SessionId) -> Result<()> {
        self.state.remove_session(&session_id)
    }

    /// Get a client's previously stored value in the db (set method) with the given key.
    pub fn get(
        &self,
        session_id: types::SessionId,
        wallet_id: String,
        key: String,
    ) -> ResponseActFuture<Option<types::RpcValue>> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            |wallet, slf: &mut Self, _| {
                slf.params
                    .worker
                    .send(worker::Get(wallet, key))
                    .flatten()
                    .map_err(From::from)
                    .and_then(|opt| match opt {
                        Some(value) => future::result(
                            serde_json::from_str(&value)
                                .map_err(internal_error)
                                .map(Some),
                        ),
                        None => future::result(Ok(None)),
                    })
                    .into_actor(slf)
            },
        );

        Box::new(f)
    }

    /// Store a client's value in the db, associated to the given key.
    pub fn set(
        &self,
        session_id: types::SessionId,
        wallet_id: String,
        key: String,
        value: types::RpcParams,
    ) -> ResponseActFuture<()> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            move |wallet, _, _| {
                fut::result(serde_json::to_string(&value).map_err(internal_error)).and_then(
                    move |value, slf: &mut Self, _| {
                        slf.params
                            .worker
                            .send(worker::Set(wallet, key, value))
                            .flatten()
                            .map_err(From::from)
                            .into_actor(slf)
                    },
                )
            },
        );

        Box::new(f)
    }

    /// Handle notifications received from the node.
    pub fn handle_block_notification(&mut self, value: types::Json) -> Result<()> {
        log::trace!("received block notification");
        let block = serde_json::from_value::<types::ChainBlock>(value).map_err(node_error)?;
        // NOTE: Possible enhancement.
        // Maybe is a good idea to use a shared reference Arc
        // instead of cloning this vector of txns if this vector
        // results to be too big, problm is that doing so conflicts
        // with the internal Cell of the txns type which cannot be
        // shared between threads.
        let block_epoch = block.block_header.beacon.checkpoint;
        // NOTE: We are calculating the hash of the block here (it's
        // just the hash of the header) since it's not memoized and
        // not doing it would imply that all threads indexing wallet
        // UTXOs call this method over and over. If calculating the
        // hash here degrades performance too much we should consider
        // to calculate it in a worker thread instead and then proceed
        // with indexing transactions.
        let block_hash = block.hash().as_ref().to_vec();
        let txns = block
            .txns
            .value_transfer_txns
            .into_iter()
            .map(|txn| txn.body)
            .collect::<Vec<_>>();

        for (id, wallet) in self.state.wallets() {
            self.params.worker.do_send(worker::IndexTxns(
                id.to_owned(),
                wallet.clone(),
                txns.clone(),
                model::BlockInfo {
                    epoch: block_epoch,
                    hash: block_hash.clone(),
                },
            ));
        }

        log::trace!("notifying balances to sessions");
        for (wallet, sink) in self.state.notifiable_wallets() {
            self.params
                .worker
                .do_send(worker::NotifyBalance(wallet, sink));
        }

        Ok(())
    }

    /// Send a transaction to witnet network using the Inventory method
    pub fn send_transaction(&self, txn: types::Transaction) -> ResponseActFuture<()> {
        let method = "inventory".to_string();
        let params = InventoryItem::Transaction(txn);

        match &self.params.client {
            Some(client) => {
                let req = types::RpcRequest::method(method)
                    .timeout(self.params.requests_timeout)
                    .params(params)
                    .expect("params failed serialization");
                let f = client
                    .send(req)
                    .flatten()
                    .map_err(From::from)
                    .map(|res| {
                        log::debug!("Inventory request result: {:?}", res);
                    })
                    .map_err(|err| {
                        log::warn!("Inventory request failed: {}", &err);
                        err
                    })
                    .into_actor(self);

                Box::new(f)
            }
            None => {
                let f = fut::err(Error::NodeNotConnected);

                Box::new(f)
            }
        }
    }

    pub fn send_vtt(
        &self,
        session_id: &types::SessionId,
        wallet_id: &str,
        transaction_hash: String,
    ) -> ResponseActFuture<()> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            move |wallet, slf: &mut Self, _| {
                slf.params
                    .worker
                    .send(worker::GetTransaction(wallet, transaction_hash))
                    .flatten()
                    .map_err(From::from)
                    .into_actor(slf)
                    .and_then(|opt_transaction, slf, _ctx| match opt_transaction {
                        Some(txn) => slf.send_transaction(txn),
                        None => {
                            let f = fut::err(validation_error(field_error(
                                "transactionId",
                                "Transaction not found",
                            )));

                            Box::new(f)
                        }
                    })
            },
        );

        Box::new(f)
    }

    pub fn send_data_req(
        &self,
        session_id: &types::SessionId,
        wallet_id: &str,
        transaction_hash: String,
    ) -> ResponseActFuture<()> {
        let f = fut::result(self.state.wallet(&session_id, &wallet_id)).and_then(
            move |wallet, slf: &mut Self, _| {
                slf.params
                    .worker
                    .send(worker::GetTransaction(wallet, transaction_hash))
                    .flatten()
                    .map_err(From::from)
                    .into_actor(slf)
                    .and_then(|opt_transaction, slf, _ctx| match opt_transaction {
                        Some(txn) => slf.send_transaction(txn),
                        None => {
                            let f = fut::err(validation_error(field_error(
                                "transactionId",
                                "Transaction not found",
                            )));

                            Box::new(f)
                        }
                    })
            },
        );

        Box::new(f)
    }
}
