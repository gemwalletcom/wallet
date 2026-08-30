package com.gemwallet.android.payment

import uniffi.gemstone.GemRecipient
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.testkit.includeGemstoneLibs
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import com.wallet.core.primitives.swap.ApprovalData
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.math.BigInteger

@RunWith(AndroidJUnit4::class)
class ConfirmInputCodecTest {

    companion object {
        init {
            includeGemstoneLibs()
        }
    }

    @Test
    fun transferPackRoundTripsThroughCoreCodec() {
        val account = mockAccount(chain = Chain.Solana)
        val original = ConfirmParams.Builder(mockAssetSolanaUSDC(), account, BigInteger("19000000"), useMaxAmount = true)
            .transfer(
                destination = GemRecipient("recipient", "recipient.sol"),
                memo = "payment-memo",
                references = listOf("reference"),
            )

        val unpacked = ConfirmParams.unpack(requireNotNull(original.pack()))

        assertTrue(unpacked is ConfirmParams.TransferParams.Token)
        val params = unpacked as ConfirmParams.TransferParams.Token
        assertEquals(original.asset, params.asset)
        assertEquals(account, params.from)
        assertEquals(BigInteger("19000000"), params.amount)
        assertEquals(GemRecipient("recipient", "recipient.sol"), params.destination)
        assertEquals("payment-memo", params.memo)
        assertEquals(listOf("reference"), params.references)
        assertTrue(params.useMaxAmount)
    }

    @Test
    fun genericPackRoundTripsThroughCoreCodec() {
        val account = mockAccount(chain = Chain.Solana)
        val approval = ApprovalData(token = "token", spender = "spender", value = "1", isUnlimited = false)
        val original = ConfirmParams.TransferParams.Generic(
            asset = mockAssetSolana(),
            from = account,
            amount = BigInteger.ZERO,
            destination = GemRecipient("merchant"),
            memo = "payment-memo",
            outputType = TransferDataOutputType.EncodedTransaction,
            outputAction = TransferDataOutputAction.Send,
            metadata = ApplicationMetadata(
                name = "Merchant",
                description = "Payment",
                url = "https://example.com",
                icon = "https://example.com/icon.png",
                source = ApplicationMetadataSource.Payment,
            ),
            data = "encoded-transaction",
            gasLimit = "21000",
            decodedTransactionType = TransactionType.Transfer,
            approval = approval,
        )

        val unpacked = ConfirmParams.unpack(requireNotNull(original.pack()))

        assertTrue(unpacked is ConfirmParams.TransferParams.Generic)
        val params = unpacked as ConfirmParams.TransferParams.Generic
        assertEquals(original.asset, params.asset)
        assertEquals(account, params.from)
        assertEquals(GemRecipient("merchant"), params.destination)
        assertEquals("payment-memo", params.memo)
        assertEquals(TransferDataOutputType.EncodedTransaction, params.outputType)
        assertTrue(params.isSendable)
        assertEquals("Merchant", params.metadata.name)
        assertEquals(ApplicationMetadataSource.Payment, params.metadata.source)
        assertEquals("encoded-transaction", params.data)
        assertEquals("21000", params.gasLimit)
        assertEquals(TransactionType.Transfer, params.decodedTransactionType)
        assertEquals(approval, params.approval)
    }

    @Test
    fun genericHexDataSurvivesCoreCodec() {
        val account = mockAccount(chain = Chain.Ethereum)
        val original = ConfirmParams.TransferParams.Generic(
            asset = mockAssetEthereum(),
            from = account,
            amount = BigInteger.ZERO,
            destination = GemRecipient("0x000000000022D473030F116dDEE9F6B43aC78BA3"),
            memo = null,
            outputType = TransferDataOutputType.Signature,
            outputAction = TransferDataOutputAction.Sign,
            metadata = ApplicationMetadata(
                name = "Dapp",
                description = "",
                url = "https://dapp.example",
                icon = "",
                source = ApplicationMetadataSource.WalletConnect,
            ),
            data = "0xa9059cbb00000000000000000000000000000000000000000000000000000000000000ff",
            gasLimit = null,
            decodedTransactionType = TransactionType.SmartContractCall,
        )

        val unpacked = ConfirmParams.unpack(requireNotNull(original.pack()))

        assertTrue(unpacked is ConfirmParams.TransferParams.Generic)
        val params = unpacked as ConfirmParams.TransferParams.Generic
        assertEquals(original.data, params.data)
        assertEquals(TransferDataOutputType.Signature, params.outputType)
        assertEquals(false, params.isSendable)
        assertEquals(null, params.approval)
    }

    @Test
    fun nativeTransferPackRoundTripsThroughCoreCodec() {
        val account = mockAccount(chain = Chain.Solana)
        val original = ConfirmParams.Builder(mockAssetSolana(), account, BigInteger.ONE)
            .transfer(destination = GemRecipient("recipient"))

        val unpacked = ConfirmParams.unpack(requireNotNull(original.pack()))

        assertTrue(unpacked is ConfirmParams.TransferParams.Native)
        val params = unpacked as ConfirmParams.TransferParams.Native
        assertEquals(original.asset, params.asset)
        assertEquals(BigInteger.ONE, params.amount)
        assertEquals(null, params.memo)
        assertEquals(false, params.useMaxAmount)
    }
}
