package com.reown.walletkit.client

import com.reown.android.Core

object Wallet {
    sealed interface Model {
        data class Error(val throwable: Throwable) : Model
        data class ConnectionState(val isAvailable: Boolean) : Model
        data class ExpiredProposal(val id: Long = 0) : Model
        data class ExpiredRequest(val topic: String = "", val id: Long = 0) : Model
        data class SessionUpdateResponse(val topic: String = "") : Model

        sealed interface SessionDelete : Model {
            data class Success(val topic: String) : SessionDelete
        }

        sealed interface SettledSessionResponse : Model {
            data class Result(val session: Session) : SettledSessionResponse
        }

        data class Session(
            val topic: String,
            val expiry: Long,
            val metaData: Core.Model.AppMetaData?,
            val namespaces: Map<String, Namespace.Session>,
            val redirect: String? = null,
        ) : Model

        data class SessionProposal(
            val name: String,
            val description: String,
            val url: String,
            val icons: List<String>,
            val requiredNamespaces: Map<String, Namespace.Proposal>,
            val optionalNamespaces: Map<String, Namespace.Proposal>,
            val proposerPublicKey: String,
            val properties: Map<String, String>? = null,
        ) : Model

        data class SessionRequest(
            val topic: String,
            val chainId: String?,
            val request: JsonRpcRequest,
        ) : Model

        data class SessionAuthenticate(
            val id: Long,
            val participant: Participant,
            val payloadParams: PayloadAuthRequestParams,
        ) : Model

        data class Participant(
            val metadata: Core.Model.AppMetaData?,
        )

        data class VerifyContext(
            val origin: String,
            val validation: Validation,
            val isScam: Boolean? = null,
        )

        enum class Validation {
            VALID,
            INVALID,
            UNKNOWN,
        }

        data class JsonRpcRequest(
            val id: Long,
            val method: String,
            val params: String,
        )

        sealed interface JsonRpcResponse {
            data class JsonRpcResult(val id: Long, val result: String) : JsonRpcResponse
            data class JsonRpcError(val id: Long, val code: Int, val message: String) : JsonRpcResponse
        }

        object Namespace {
            data class Proposal(
                val chains: List<String>? = null,
            )

            data class Session(
                val chains: List<String>? = null,
                val methods: List<String>,
                val events: List<String>,
                val accounts: List<String>,
            )
        }

        data class PayloadAuthRequestParams(
            val chains: List<String>,
            val methods: List<String> = emptyList(),
        )

        data class Cacao(
            val signature: Signature? = null,
        ) {
            data class Signature(
                val t: String,
                val s: String,
            )
        }
    }

    object Params {
        data class Init(val core: Any)
        data class Pair(val uri: String)
        data class Ping(val topic: String)
        data class SessionDisconnect(val sessionTopic: String)
        data class SessionRequestResponse(
            val sessionTopic: String,
            val jsonRpcResponse: Model.JsonRpcResponse,
        )

        data class SessionApprove(
            val proposerPublicKey: String,
            val namespaces: Map<String, Model.Namespace.Session>,
            val properties: Map<String, String>,
        )

        data class SessionReject(
            val proposerPublicKey: String,
            val reason: String,
        )

        data class ApproveSessionAuthenticate(
            val id: Long,
            val auths: List<Model.Cacao>,
        )

        data class RejectSessionAuthenticate(
            val id: Long,
            val reason: String,
        )

        data class FormatAuthMessage(
            val payloadParams: Model.PayloadAuthRequestParams,
            val issuer: String,
        )
    }
}
