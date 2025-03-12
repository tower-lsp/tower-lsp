macro_rules! rpc {
    // Entrypoint
    (

        $(#[doc = $trait_docs:literal])*
        pub trait $name:ident: ($($bounds:tt)+) {
            $(
                $(#[doc = $method_docs:literal])*
                #[rpc(name = $rpc_name:literal)]
                async fn $rpc_method:ident(&self$(, $rpc_params_ident:ident: $rpc_params:ty)? $(,)?) $(-> $rpc_result:ty)? $($body:block)? $(;)?
            )+
        }
    ) => {
        $(#[doc = $trait_docs])*
        pub trait $name: $($bounds)+ {
            $(
                rpc!(@method
                    $(#[doc = $method_docs])*,
                    $rpc_method,
                    $($rpc_params_ident: $rpc_params)?,
                    $($rpc_result)?,
                    $($body)?
                );
            )+
        }

        mod generated {
            use crate::jsonrpc::Router;
            use crate::service::{layers, Client, Pending, ServerState, ExitedError};
            use lsp_types::*;
            use std::sync::Arc;
            use super::LanguageServer;

            pub(crate) fn register_lsp_methods<S>(
                mut router: Router<S, ExitedError>,
                state: Arc<ServerState>,
                pending: Arc<Pending>,
                client: Client,
            ) -> Router<S, ExitedError>
            where
                S: LanguageServer,
            {
                $(
                    rpc!(@register
                        $rpc_name,
                        $rpc_method,
                        router,
                        state,
                        pending
                    );
                )+

                router.method(
                    "exit",
                    // this closure is never called
                    |_: &S| std::future::ready(()),
                    layers::Exit::new(state.clone(), pending.clone(), client.clone()),
                );

                router.method(
                    "$/cancelRequest",
                    move |_: &S, params: CancelParams| {
                        pending.cancel(&params.id.into());
                        std::future::ready(())
                    },
                    tower::layer::util::Identity::new(),
                );

                router
            }
        }
    };

    // `method` fragment: transform methods signatures to add `Send` bound
    (@method $(#[doc = $method_docs:literal])*, $rpc_method:ident, /* empty params */, $rpc_result:ty, /* empty body */) => {
        $(#[doc = $method_docs])*
        fn $rpc_method(&self) -> impl ::std::future::Future<Output = $rpc_result> + Send;
    };
    (@method $(#[doc = $method_docs:literal])*, $rpc_method:ident, $rpc_params_ident:ident: $rpc_params:ty, $rpc_result:ty, /* empty body */) => {
        $(#[doc = $method_docs])*
        fn $rpc_method(&self, $rpc_params_ident: $rpc_params) -> impl ::std::future::Future<Output = $rpc_result> + Send;
    };
    (@method $(#[doc = $method_docs:literal])*, $rpc_method:ident, $rpc_params_ident:ident: $rpc_params:ty, $rpc_result:ty, $body:block) => {
        $(#[doc = $method_docs])*
        fn $rpc_method(&self, $rpc_params_ident: $rpc_params) -> impl ::std::future::Future<Output = $rpc_result> + Send { async $body }
    };
    (@method $(#[doc = $method_docs:literal])*, $rpc_method:ident, $rpc_params_ident:ident: $rpc_params:ty, /* empty result */, /* empty body */) => {
        $(#[doc = $method_docs])*
        fn $rpc_method(&self, $rpc_params_ident: $rpc_params) -> impl ::std::future::Future<Output = ()> + Send;
    };
    (@method $(#[doc = $method_docs:literal])*, $rpc_method:ident, $rpc_params_ident:ident: $rpc_params:ty, /* empty result */, $body:block) => {
        $(#[doc = $method_docs])*
        fn $rpc_method(&self, $rpc_params_ident: $rpc_params) -> impl ::std::future::Future<Output = ()> + Send { async $body }
    };

    // `register` fragment: add the method to the tower router
    (@register $rpc_name:literal, initialize, $router:ident, $state:ident, $pending:ident) => {
        $router.method(
            "initialize",
            S::initialize,
            layers::Initialize::new($state.clone(), $pending.clone()),
        )
    };
    (@register $rpc_name:literal, shutdown, $router:ident, $state:ident, $pending:ident) => {
        $router.method(
            "shutdown",
            S::shutdown,
            layers::Shutdown::new($state.clone(), $pending.clone()),
        )
    };
    (@register $rpc_name:literal, $rpc_method:ident, $router:ident, $state:ident, $pending:ident) => {
        $router.method(
            $rpc_name,
            S::$rpc_method,
            layers::Normal::new($state.clone(), $pending.clone()),
        )
    };
}
