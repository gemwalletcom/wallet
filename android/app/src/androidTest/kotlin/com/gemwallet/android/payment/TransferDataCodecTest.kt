package com.gemwallet.android.payment

import com.gemwallet.android.ext.toPrimitives
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.has0xPrefix
import com.gemwallet.android.domains.confirm.transfer
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.includeGemstoneLibs
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.ApprovalData
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransferDataExtra
import java.math.BigInteger
import com.gemwallet.android.domains.confirm.unpackTransferData

@RunWith(AndroidJUnit4::class)
class TransferDataCodecTest {

    companion object {
        init {
            includeGemstoneLibs()
        }
    }

    private fun roundTrip(transfer: GemTransferData): GemTransferData =
        requireNotNull(unpackTransferData(requireNotNull(transfer.pack())))

    private fun generic(
        asset: Asset,
        recipient: GemRecipient,
        metadata: ApplicationMetadata,
        data: String,
        outputType: TransferDataOutputType,
        outputAction: TransferDataOutputAction,
        transactionType: TransactionType,
        gasLimit: BigInteger? = null,
        approval: ApprovalData? = null,
    ) = GemTransferData(
        inputType = TransactionInputType.Generic(
            asset = asset.toGem(),
            metadata = metadata.toGem(),
            extra = TransferDataExtra(
                to = recipient.address,
                gasLimit = gasLimit,
                gasPrice = null,
                data = data.toTransactionData(),
                outputType = outputType.toGem(),
                outputAction = outputAction.toGem(),
                transactionType = transactionType.toGem(),
                approval = approval,
            ),
        ),
        recipient = recipient,
        value = BigInteger.ZERO,
    )

    @Test
    fun transferPackRoundTripsThroughCoreCodec() {
        val asset = mockAssetSolanaUSDC()
        val original = GemTransferData(
            inputType = TransactionInputType.transfer(asset),
            recipient = GemRecipient(address = "recipient", name = "recipient.sol", memo = "payment-memo", references = listOf("reference")),
            value = BigInteger("19000000"),
            useMaxAmount = true,
        )

        val transfer = roundTrip(original)

        assertTrue(transfer.inputType is TransactionInputType.Transfer)
        assertEquals(asset, transfer.asset)
        assertEquals(BigInteger("19000000"), transfer.value)
        assertEquals("recipient", transfer.recipient.address)
        assertEquals("recipient.sol", transfer.recipient.name)
        assertEquals("payment-memo", transfer.recipient.memo)
        assertEquals(listOf("reference"), transfer.recipient.references)
        assertTrue(transfer.useMaxAmount)
    }

    @Test
    fun genericPackRoundTripsThroughCoreCodec() {
        val asset = mockAssetSolana()
        val approval = ApprovalData(token = "token", spender = "spender", value = BigInteger.ONE, isUnlimited = false)
        val original = generic(
            asset = asset,
            recipient = GemRecipient(address = "merchant", memo = "payment-memo"),
            metadata = ApplicationMetadata(
                name = "Merchant",
                description = "Payment",
                url = "https://example.com",
                icon = "https://example.com/icon.png",
                source = ApplicationMetadataSource.Payment,
            ),
            data = "encoded-transaction",
            outputType = TransferDataOutputType.EncodedTransaction,
            outputAction = TransferDataOutputAction.Send,
            transactionType = TransactionType.Transfer,
            gasLimit = BigInteger("21000"),
            approval = approval,
        )

        val transfer = roundTrip(original)
        val assetId = transfer.asset.id
        val generic = transfer.inputType as TransactionInputType.Generic
        val metadata = generic.metadata.toPrimitives()

        assertEquals(asset.id, assetId)
        assertEquals("merchant", transfer.recipient.address)
        assertEquals("payment-memo", transfer.recipient.memo)
        assertEquals(TransferDataOutputType.EncodedTransaction, generic.extra.outputType.toPrimitives())
        assertEquals(TransferDataOutputAction.Send, generic.extra.outputAction.toPrimitives())
        assertEquals("Merchant", metadata.name)
        assertEquals(ApplicationMetadataSource.Payment, metadata.source)
        assertEquals("encoded-transaction", String(requireNotNull(generic.extra.data)))
        assertEquals(BigInteger("21000"), generic.extra.gasLimit)
        assertEquals(TransactionType.Transfer, generic.extra.transactionType.toPrimitives())
        assertEquals(approval, generic.extra.approval)
    }

    @Test
    fun genericHexDataSurvivesCoreCodec() {
        val data = "0xa9059cbb00000000000000000000000000000000000000000000000000000000000000ff"
        val original = generic(
            asset = mockAssetEthereum(),
            recipient = GemRecipient("0x000000000022D473030F116dDEE9F6B43aC78BA3"),
            metadata = ApplicationMetadata(
                name = "Dapp",
                description = "",
                url = "https://dapp.example",
                icon = "",
                source = ApplicationMetadataSource.WalletConnect,
            ),
            data = data,
            outputType = TransferDataOutputType.Signature,
            outputAction = TransferDataOutputAction.Sign,
            transactionType = TransactionType.SmartContractCall,
        )

        val generic = roundTrip(original).inputType as TransactionInputType.Generic

        assertArrayEquals(data.toTransactionData(), generic.extra.data)
        assertEquals(TransferDataOutputType.Signature, generic.extra.outputType.toPrimitives())
        assertEquals(TransferDataOutputAction.Sign, generic.extra.outputAction.toPrimitives())
        assertEquals(null, generic.extra.approval)
    }

    @Test
    fun nativeTransferPackRoundTripsThroughCoreCodec() {
        val asset = mockAssetSolana()
        val original = GemTransferData(
            inputType = TransactionInputType.transfer(asset),
            recipient = GemRecipient("recipient"),
            value = BigInteger.ONE,
        )

        val transfer = roundTrip(original)

        assertTrue(transfer.inputType is TransactionInputType.Transfer)
        assertEquals(asset, transfer.asset)
        assertEquals(BigInteger.ONE, transfer.value)
        assertEquals(null, transfer.recipient.memo)
        assertEquals(false, transfer.useMaxAmount)
    }
}

private fun String.toTransactionData(): ByteArray =
    if (has0xPrefix()) runCatching { fromHex() }.getOrElse { toByteArray() } else toByteArray()
