package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.repositories.assets.RecentAssetsService
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.blockchain.gemstone.toGemSignerFee
import com.gemwallet.android.domains.confirm.toTransferData
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.Transaction
import uniffi.gemstone.GemPendingTransactionInput
import uniffi.gemstone.GemTransferService
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemSignedTransaction
import uniffi.gemstone.GemSignerError
import uniffi.gemstone.GemstoneException

class ConfirmTransactionImpl(
    private val passwordStore: PasswordStore,
    private val signTransactionOperator: GemSignTransactionOperator,
    private val confirmService: GemConfirmServiceInterface,
    private val createTransactionsCase: CreateTransaction,
    private val recentAssetsService: RecentAssetsService,
) : ConfirmTransaction {
    override suspend fun invoke(
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
        scope: CoroutineScope,
        simulation: SimulationResult?,
    ): String {
        val account = assetInfo.owner ?: throw ConfirmError.TransactionIncorrect

        val signedTransactions = sign(signerParams, session)
        if (signedTransactions.isEmpty()) throw ConfirmError.SignFail

        if ((signerParams.input as? ConfirmParams.TransferParams.Generic)?.isSendable == false) {
            return signedTransactions.first().data
        }

        signedTransactions.forEach { signedTransaction ->
            val approval = signerParams.input.approvalData(signedTransaction.transactionType.decodeJson())
            if (approval != null && approval.value.toBigIntegerOrNull() == null) {
                throw ConfirmError.TransactionIncorrect
            }
        }

        val hashes = try {
            confirmService.broadcast(signerParams.input.toDto(), signedTransactions)
        } catch (error: GemConfirmException.Broadcast) {
            addTransactions(error.hashes, signedTransactions, signerParams, session, account, simulation)
            throw error
        }
        addTransactions(hashes, signedTransactions, signerParams, session, account, simulation)
        scope.launch(Dispatchers.IO) { addRecent(assetInfo, signerParams.input) }

        return hashes.last()
    }

    private suspend fun addTransactions(
        hashes: List<String>,
        signedTransactions: List<GemSignedTransaction>,
        signerParams: SignerParams,
        session: Session,
        account: Account,
        simulation: SimulationResult?,
    ) {
        for ((index, transactionHash) in hashes.withIndex()) {
            val transaction = try {
                GemTransferService().pendingTransaction(
                    GemPendingTransactionInput(
                        sender = account.address,
                        transfer = signerParams.input.toTransferData(),
                        value = signerParams.finalAmount.toString(),
                        transactionType = signedTransactions[index].transactionType,
                        hash = transactionHash,
                        fee = signerParams.fee().toGemSignerFee(),
                        networkFee = signerParams.fee().amount.toString(),
                        metadata = signerParams.data().metadata,
                        simulation = simulation?.toJson(),
                        transactionIndex = index.toUInt(),
                        transactionCount = signedTransactions.size.toUInt(),
                    )
                )
            } catch (_: GemstoneException) {
                throw ConfirmError.TransactionIncorrect
            } ?: continue
            createTransactionsCase.createTransaction(session.wallet.id, transaction.decodeJson<Transaction>())
        }
    }

    private suspend fun sign(
        signerParams: SignerParams,
        session: Session,
    ): List<GemSignedTransaction> {
        return try {
            signTransactionOperator(
                session.wallet,
                signerParams,
                passwordStore.getPassword(session.wallet.id.id),
            )
        } catch (error: GemstoneException.SignerException) {
            throw error.error.toConfirmError(signerParams.input.assetId.chain)
        } catch (_: Throwable) {
            throw ConfirmError.SignFail
        }
    }

    private suspend fun addRecent(assetInfo: AssetInfo, request: ConfirmParams) {
        val walletId = assetInfo.walletId?.id ?: return
        val type = when (request) {
            is ConfirmParams.SwapParams -> RecentType.Swap
            is ConfirmParams.TransferParams -> RecentType.Send
            else -> return
        }
        val toAssetId = if (request is ConfirmParams.SwapParams) {
            request.toAsset.id
        } else {
            null
        }
        try {
            recentAssetsService.addRecentActivity(assetInfo.id(), walletId, type, toAssetId)
        } catch (_: Throwable) {}
    }
}

internal fun GemSignerError.toConfirmError(chain: Chain): ConfirmError = when (this) {
    GemSignerError.DustThreshold -> ConfirmError.DustThreshold(chain)
    is GemSignerError.InvalidInput,
    is GemSignerError.SigningError,
    GemSignerError.InsufficientFunds,
    GemSignerError.SwapValueBelowMinimum -> ConfirmError.SignFail
}
