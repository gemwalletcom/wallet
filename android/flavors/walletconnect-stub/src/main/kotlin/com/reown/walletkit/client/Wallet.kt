package com.reown.walletkit.client

import com.reown.android.Core

object Wallet {
    interface Model {
        class Error(val throwable: Throwable) : Model
        class ConnectionState(val isAvailable: Boolean) : Model
        class ExpiredProposal : Model
        class ExpiredRequest(val topic: String = "", val id: Long = 0) : Model
        class SessionUpdateResponse : Model

        interface SessionDelete : Model {
            class Success(val topic: String) : SessionDelete
        }

        interface SettledSessionResponse : Model {
            class Result(val session: Session) : SettledSessionResponse
        }

        class Session(
            val topic: String,
            val expiry: Long,
            val metaData: Core.Model.AppMetaData?,
            val namespaces: Map<String, Namespace.Session>,
            val redirect: String? = null,
        ) : Model

        class SessionProposal(
            val name: String,
            val description: String,
            val url: String,
            val icons: List<String>,
            val requiredNamespaces: Map<String, Namespace.Proposal>,
            val optionalNamespaces: Map<String, Namespace.Proposal>,
            val proposerPublicKey: String,
            val properties: Map<String, String>? = null,
        ) : Model

        class SessionRequest(
            val topic: String,
            val chainId: String?,
            val request: JsonRpcRequest,
        ) : Model

        class SessionAuthenticate(
            val id: Long,
            val participant: Participant,
            val payloadParams: PayloadAuthRequestParams,
        ) : Model

        class Participant(val metadata: Core.Model.AppMetaData?)

        class VerifyContext(
            val origin: String,
            val validation: Validation,
            val isScam: Boolean? = null,
        )

        enum class Validation { VALID, INVALID, UNKNOWN }

        class JsonRpcRequest(
            val id: Long,
            val method: String,
            val params: String,
        )

        interface JsonRpcResponse {
            class JsonRpcResult(id: Long, result: String) : JsonRpcResponse
            class JsonRpcError(id: Long, code: Int, message: String) : JsonRpcResponse
        }

        object Namespace {
            class Proposal(val chains: List<String>? = null)

            class Session(
                val chains: List<String>? = null,
                methods: List<String>,
                events: List<String>,
                val accounts: List<String>,
            )
        }

        class PayloadAuthRequestParams(
            val chains: List<String>,
            methods: List<String> = emptyList(),
        )

        class Cacao {
            class Signature(t: String, s: String)
        }
    }

    object Params {
        class Init(core: Any)
        class Pair(uri: String)
        class Ping(topic: String)
        class SessionDisconnect(sessionTopic: String)
        class SessionRequestResponse(sessionTopic: String, jsonRpcResponse: Model.JsonRpcResponse)
        class SessionApprove(
            proposerPublicKey: String,
            namespaces: Map<String, Model.Namespace.Session>,
            properties: Map<String, String>,
        )
        class SessionReject(proposerPublicKey: String, reason: String)
        class ApproveSessionAuthenticate(id: Long, auths: List<Model.Cacao>)
        class RejectSessionAuthenticate(id: Long, reason: String)
        class FormatAuthMessage(payloadParams: Model.PayloadAuthRequestParams, issuer: String)
    }
}
