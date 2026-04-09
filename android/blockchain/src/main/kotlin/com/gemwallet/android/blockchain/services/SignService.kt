package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.clients.SignClient
import com.gemwallet.android.blockchain.gemstone.toGemSignerInput
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.model.ChainSignData
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Fee
import com.wallet.core.primitives.Chain
import com.gemwallet.android.ext.toChainType
import com.wallet.core.primitives.ChainType
import uniffi.gemstone.GemChainSigner
import java.math.BigInteger

class SignService : SignClient {

    override suspend fun signMessage(
        chain: Chain,
        input: ByteArray,
        privateKey: ByteArray
    ): ByteArray {
        return GemChainSigner(chain.string).signMessage(input, privateKey).toByteArray()
    }

    override suspend fun signTypedMessage(
        chain: Chain,
        input: ByteArray,
        privateKey: ByteArray
    ): String {
        return GemChainSigner(chain.string).signMessage(input, privateKey)
    }

    override suspend fun signActivate(
        params: ConfirmParams.Activate,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return listOf(
            getSigner(params).signAccountAction(data, privateKey).toByteArray()
        )
    }

    override suspend fun signDelegate(
        params: ConfirmParams.Stake.DelegateParams,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signStake(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signFreeze(
        params: ConfirmParams.Stake.Freeze,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signStake(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signGenericTransfer(
        params: ConfirmParams.TransferParams.Generic,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return listOf(getSigner(params).signData(data, privateKey).toByteArray())
    }

    override suspend fun signNativeTransfer(
        params: ConfirmParams.TransferParams.Native,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return listOf(
            getSigner(params).signTransfer(data, privateKey).toByteArray()
        )
    }

    override suspend fun signNft(
        params: ConfirmParams.NftParams,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return listOf(
            getSigner(params).signNftTransfer(data, privateKey).toByteArray()
        )
    }

    override suspend fun signPerpetualClose(
        params: ConfirmParams.PerpetualParams.Close,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signPerpetual(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signPerpetualModify(
        params: ConfirmParams.PerpetualParams.Modify,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signPerpetual(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signPerpetualOpen(
        params: ConfirmParams.PerpetualParams.Open,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signPerpetual(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signRedelegate(
        params: ConfirmParams.Stake.RedelegateParams,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signStake(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signRewards(
        params: ConfirmParams.Stake.RewardsParams,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signStake(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signSwap(
        params: ConfirmParams.SwapParams,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signSwap(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signTokenApproval(
        params: ConfirmParams.TokenApprovalParams,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return listOf(getSigner(params).signTokenApproval(data, privateKey).toByteArray())
    }

    override suspend fun signTokenTransfer(
        params: ConfirmParams.TransferParams.Token,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return listOf(getSigner(params).signTokenTransfer(data, privateKey).toByteArray())
    }

    override suspend fun signUndelegate(
        params: ConfirmParams.Stake.UndelegateParams,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signStake(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signUnfreeze(
        params: ConfirmParams.Stake.Unfreeze,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return getSigner(params).signStake(data, privateKey).map { it.toByteArray() }
    }

    override suspend fun signWithdraw(
        params: ConfirmParams.Stake.WithdrawParams,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
        privateKey: ByteArray
    ): List<ByteArray> {
        val data = buildSignerInput(
            params = params,
            chainData = chainData,
            finalAmount = finalAmount,
            fee = fee,
        )
        return listOf(getSigner(params).signWithdrawal(data, privateKey).toByteArray())
    }
    override fun supported(chain: Chain): Boolean {
        return when (chain.toChainType()) {
            ChainType.Ethereum,
            ChainType.Aptos,
            ChainType.Sui,
            ChainType.HyperCore,
            ChainType.Near -> true
            else -> false
        }
    }

    private fun buildSignerInput(
        params: ConfirmParams,
        chainData: ChainSignData,
        finalAmount: BigInteger,
        fee: Fee,
    ) = params.toGemSignerInput(
        chainData = chainData,
        finalAmount = finalAmount,
        fee = fee,
    )

    private fun getSigner(params: ConfirmParams) = GemChainSigner(params.asset.chain.string)
}
