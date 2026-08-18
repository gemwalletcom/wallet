package com.gemwallet.android.payment

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ext.request
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.PaymentDestination
import com.gemwallet.android.model.PaymentTransfer
import com.gemwallet.android.testkit.includeGemstoneLibs
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSmartChain
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockAssetXrp
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.paymentDecodeUrl
import java.math.BigInteger

private const val BITCOIN_ADDRESS = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
private const val SOLANA_ADDRESS = "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5"
private const val RIPPLE_ADDRESS = "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh"
private const val EVM_ADDRESS = "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326"
private const val USDC_TOKEN_ID = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"

@RunWith(AndroidJUnit4::class)
class PaymentTransferTest {

    companion object {
        init {
            includeGemstoneLibs()
        }
    }

    private val bitcoin = mockAssetInfo(asset = mockAsset())
    private val solana = mockAssetInfo(asset = mockAssetSolana())
    private val ripple = mockAssetInfo(asset = mockAssetXrp())
    private val usdc = mockAssetInfo(asset = mockAssetSolanaUSDC())

    private fun decode(url: String): PaymentRequest =
        requireNotNull(paymentDecodeUrl(url).toPrimitives().request) { "not a payment request: $url" }

    private fun destination(assetInfo: AssetInfo, url: String): PaymentDestination.Transfer =
        PaymentTransfer(assetInfo).destination(decode(url))

    @Test
    fun destination_confirm() {
        val confirm = destination(bitcoin, "bitcoin:$BITCOIN_ADDRESS?amount=0.0001")

        assertTrue("expected a confirmable transfer, got $confirm", confirm is PaymentDestination.Confirm)
        val params = (confirm as PaymentDestination.Confirm).params
        assertEquals(BigInteger("10000"), params.amount)
        assertEquals(BITCOIN_ADDRESS, params.destination()?.address)
        assertTrue(params is ConfirmParams.TransferParams.Native)
    }

    @Test
    fun destination_recipient() {
        val recipient = destination(bitcoin, "bitcoin:$BITCOIN_ADDRESS")

        assertTrue("expected the recipient screen, got $recipient", recipient is PaymentDestination.Recipient)
        assertEquals(BITCOIN_ADDRESS, (recipient as PaymentDestination.Recipient).request.address)
        assertEquals(null, recipient.request.amount)
    }

    @Test
    fun destination_otherAsset() {
        val url = "solana:$SOLANA_ADDRESS?amount=1&spl-token=$USDC_TOKEN_ID"

        assertEquals(PaymentDestination.Unsupported, destination(solana, url))

        val confirm = destination(usdc, url)

        assertTrue("expected USDC to confirm, got $confirm", confirm is PaymentDestination.Confirm)
        assertEquals(BigInteger("1000000"), (confirm as PaymentDestination.Confirm).params.amount)
    }

    @Test
    fun destination_xrpDestinationTag_confirms() {
        val confirm = destination(ripple, "ripple:$RIPPLE_ADDRESS?amount=10&dt=12345")

        assertTrue("an exact tagged payment must confirm, got $confirm", confirm is PaymentDestination.Confirm)
        val params = (confirm as PaymentDestination.Confirm).params
        assertEquals(BigInteger("10000000"), params.amount)
        assertEquals(RIPPLE_ADDRESS, params.destination()?.address)
        assertEquals("12345", params.memo())
    }

    @Test
    fun destination_belowSmallestUnit() {
        val recipient = destination(usdc, "solana:$SOLANA_ADDRESS?amount=0.0000001&spl-token=$USDC_TOKEN_ID")

        assertTrue("a seventh decimal is not signable as USDC, got $recipient", recipient is PaymentDestination.Recipient)
    }

    @Test
    fun from_walletAssets() {
        val ethereum = mockAssetInfo(asset = mockAssetEthereum())
        val smartChain = mockAssetInfo(asset = mockAssetSmartChain())
        val assets = listOf(bitcoin, ethereum, smartChain)

        assertEquals(
            PaymentDestination.SelectAsset(decode(EVM_ADDRESS), listOf(Chain.Ethereum, Chain.SmartChain)),
            PaymentDestination.from(decode(EVM_ADDRESS), assets),
        )

        val confirm = PaymentDestination.from(decode("bitcoin:$BITCOIN_ADDRESS?amount=0.0001"), assets)
        assertTrue("one payable asset must go straight to confirm, got $confirm", confirm is PaymentDestination.Confirm)

        assertEquals(
            PaymentDestination.Unsupported,
            PaymentDestination.from(decode("ripple:$RIPPLE_ADDRESS"), assets),
        )
    }
}
