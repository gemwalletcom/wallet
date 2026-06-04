package com.gemwallet.android.blockchain.clients

import uniffi.gemstone.GemTransactionLoadMetadata
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Fee
import com.wallet.core.primitives.Chain
import java.math.BigInteger

interface SignClient {

    suspend fun signMessage(
        chain: Chain,
        input: ByteArray,
        privateKey: ByteArray,
    ): ByteArray = byteArrayOf()

    suspend fun signTypedMessage(chain: Chain, input: ByteArray, privateKey: ByteArray): String = ""

    suspend fun signGenericTransfer(
        params: ConfirmParams.TransferParams.Generic,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signNativeTransfer(
        params: ConfirmParams.TransferParams.Native,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signTokenTransfer(
        params: ConfirmParams.TransferParams.Token,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signSwap(
        params: ConfirmParams.SwapParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signTokenApproval(
        params: ConfirmParams.TokenApprovalParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signDelegate(
        params: ConfirmParams.Stake.DelegateParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signUndelegate(
        params: ConfirmParams.Stake.UndelegateParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signRedelegate(
        params: ConfirmParams.Stake.RedelegateParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signRewards(
        params: ConfirmParams.Stake.RewardsParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signWithdraw(
        params: ConfirmParams.Stake.WithdrawParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signActivate(
        params: ConfirmParams.Activate,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signNft(
        params: ConfirmParams.NftParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signFreeze(
        params: ConfirmParams.Stake.Freeze,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signUnfreeze(
        params: ConfirmParams.Stake.Unfreeze,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()

    suspend fun signPerpetual(
        params: ConfirmParams.PerpetualParams,
        metadata: GemTransactionLoadMetadata,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray,
    ): List<ByteArray> = emptyList()
}
