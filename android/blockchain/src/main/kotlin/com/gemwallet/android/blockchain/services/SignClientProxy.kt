package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.clients.SignClient
import uniffi.gemstone.GemTransactionLoadMetadata
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.Fee
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemSwapQuoteDataType
import java.math.BigInteger

class SignClientProxy(
    private val client: SignClient,
) {

    suspend fun signMessage(
        chain: Chain,
        input: ByteArray,
        privateKey: ByteArray
    ): ByteArray {
        return client.signMessage(chain, input, privateKey)
    }

    suspend fun signTypedMessage(
        chain: Chain,
        input: ByteArray,
        privateKey: ByteArray
    ): String {
        return client.signTypedMessage(chain, input, privateKey)
    }

    suspend fun signTransaction(
        params: SignerParams,
        privateKey: ByteArray,
    ): List<ByteArray> {
        val input = params.input
        val data = params.data()
        val fee = data.fee
        val metadata = data.metadata
        return when (input) {
            is ConfirmParams.Stake.DelegateParams -> client.signDelegate(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.Stake.RedelegateParams -> client.signRedelegate(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.Stake.RewardsParams -> client.signRewards(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.Stake.UndelegateParams -> client.signUndelegate(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.Stake.WithdrawParams -> client.signWithdraw(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.Stake.Freeze -> client.signFreeze(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.Stake.Unfreeze -> client.signUnfreeze(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.SwapParams -> signSwap(params.input as ConfirmParams.SwapParams, metadata, params.finalAmount, fee, privateKey, client)
            is ConfirmParams.TokenApprovalParams -> client.signTokenApproval(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.TransferParams.Generic -> client.signGenericTransfer(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.TransferParams.Native -> client.signNativeTransfer(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.TransferParams.Token -> client.signTokenTransfer(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.Activate -> client.signActivate(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.NftParams -> client.signNft(input, metadata, params.finalAmount, fee, privateKey)
            is ConfirmParams.PerpetualParams -> client.signPerpetual(input, metadata, params.finalAmount, fee, privateKey)
        }
    }

    private suspend fun signSwap(
        params: ConfirmParams.SwapParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
        client: SignClient,
    ): List<ByteArray> {

        return when (params.dataType) {
            GemSwapQuoteDataType.CONTRACT -> client.signSwap(params, metadata, finalAmount, fee, privateKey)
            GemSwapQuoteDataType.TRANSFER -> signSwapTransfer(
                params,
                metadata,
                finalAmount,
                fee,
                client,
                privateKey,
            )
        }
    }

    private suspend fun signSwapTransfer(
        params: ConfirmParams.SwapParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        client: SignClient,
        privateKey: ByteArray,
    ): List<ByteArray> {
        val memo = params.memo()
        val destinationAddress = params.toAddress
        val transferParams = ConfirmParams.Builder(
            asset = params.fromAsset,
            from = params.from,
            amount = finalAmount,
            useMaxAmount = params.useMaxAmount,
        )
            .transfer(
                destination = DestinationAddress(destinationAddress),
                memo = memo,
            )
        return when (transferParams) {
            is ConfirmParams.TransferParams.Native -> client.signNativeTransfer(
                transferParams,
                metadata,
                finalAmount,
                fee,
                privateKey,
            )
            is ConfirmParams.TransferParams.Token -> client.signTokenTransfer(
                transferParams,
                metadata,
                finalAmount,
                fee,
                privateKey,
            )
            else -> throw IllegalArgumentException("Not sound signer")
        }
    }
}
