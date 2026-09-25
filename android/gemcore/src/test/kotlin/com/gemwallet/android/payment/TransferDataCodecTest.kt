package com.gemwallet.android.payment

import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.domains.confirm.unpackTransferData
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.has0xPrefix
import com.gemwallet.android.testkit.mockApplicationMetadata
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockTransferDataExtra
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.ApprovalData
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

class TransferDataCodecTest {

    private fun roundTrip(transfer: GemTransferData): GemTransferData = requireNotNull(unpackTransferData(requireNotNull(transfer.pack())))

    @Test
    fun transferPackRoundTripsThroughCoreCodec() {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)
        val original =
            mockGemTransferData(
                inputType = TransactionInputType.Transfer(asset.toGem()),
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
        val asset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9)
        val approval = ApprovalData(token = "token", spender = "spender", value = BigInteger.ONE, isUnlimited = false)
        val original = mockGemTransferData(
            inputType = TransactionInputType.Generic(
                asset = asset.toGem(),
                metadata = mockApplicationMetadata(name = "Merchant", source = ApplicationMetadataSource.WalletConnect).toGem(),
                extra = mockTransferDataExtra(
                    to = "merchant",
                    gasLimit = BigInteger("21000"),
                    data = "encoded-transaction".toTransactionData(),
                    outputType = TransferDataOutputType.EncodedTransaction.toGem(),
                    outputAction = TransferDataOutputAction.Send.toGem(),
                    transactionType = TransactionType.Transfer.toGem(),
                    approval = approval,
                ),
            ),
            recipient = GemRecipient(address = "merchant", memo = "payment-memo"),
            value = BigInteger.ONE,
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
        assertEquals(ApplicationMetadataSource.WalletConnect, metadata.source)
        assertEquals("encoded-transaction", String(requireNotNull(generic.extra.data)))
        assertEquals(BigInteger("21000"), generic.extra.gasLimit)
        assertEquals(TransactionType.Transfer, generic.extra.transactionType.toPrimitives())
        assertEquals(approval, generic.extra.approval)
    }

    @Test
    fun genericHexDataSurvivesCoreCodec() {
        val data = "0xa9059cbb00000000000000000000000000000000000000000000000000000000000000ff"
        val original = mockGemTransferData(
            inputType = TransactionInputType.Generic(
                asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18).toGem(),
                metadata = mockApplicationMetadata().toGem(),
                extra = mockTransferDataExtra(
                    to = "0x000000000022D473030F116dDEE9F6B43aC78BA3",
                    data = data.toTransactionData(),
                    outputType = TransferDataOutputType.Signature.toGem(),
                    outputAction = TransferDataOutputAction.Sign.toGem(),
                    transactionType = TransactionType.SmartContractCall.toGem(),
                ),
            ),
            recipient = GemRecipient("0x000000000022D473030F116dDEE9F6B43aC78BA3"),
            value = BigInteger.ONE,
        )

        val generic = roundTrip(original).inputType as TransactionInputType.Generic

        assertArrayEquals(data.toTransactionData(), generic.extra.data)
        assertEquals(TransferDataOutputType.Signature, generic.extra.outputType.toPrimitives())
        assertEquals(TransferDataOutputAction.Sign, generic.extra.outputAction.toPrimitives())
        assertEquals(null, generic.extra.approval)
    }

    @Test
    fun nativeTransferPackRoundTripsThroughCoreCodec() {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9)
        val original = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE)

        val transfer = roundTrip(original)

        assertTrue(transfer.inputType is TransactionInputType.Transfer)
        assertEquals(asset, transfer.asset)
        assertEquals(BigInteger.ONE, transfer.value)
        assertEquals(null, transfer.recipient.memo)
        assertEquals(false, transfer.useMaxAmount)
    }
}

private fun String.toTransactionData(): ByteArray = if (has0xPrefix()) runCatching { fromHex() }.getOrElse { toByteArray() } else toByteArray()
