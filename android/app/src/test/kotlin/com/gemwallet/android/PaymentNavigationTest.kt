package com.gemwallet.android

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.unpackTransferData
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.navigation.routes.AmountRoute
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.PaymentVerificationRoute
import com.gemwallet.android.ui.navigation.routes.RecipientInputRoute
import com.gemwallet.android.ui.navigation.routes.SendSelectRoute
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemPaymentServiceInterface
import uniffi.gemstone.GemPaymentTarget
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.Payment
import uniffi.gemstone.PaymentLink
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

class PaymentNavigationTest {

    private val payment = Payment.Link(PaymentLink.SolanaPay("https://example.com/pay"))
    private val getSession = mockk<GetSession> { every { this@mockk() } returns MutableStateFlow(mockSession()) }

    private fun navigation(target: GemPaymentTarget): PaymentNavigation {
        val paymentService = mockk<GemPaymentServiceInterface> {
            coEvery { prepare(any(), any()) } returns target
        }
        return PaymentNavigation(getSession, paymentService)
    }

    private fun recipient() = GemPaymentRecipient(GemRecipient("address", null, null, emptyList()), null)

    @Test
    fun `a prepared transfer confirms`() = runTest {
        val transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(mockAsset().toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE)

        val route = navigation(GemPaymentTarget.Confirm(transfer)).routes(payment).single() as ConfirmRoute

        assertEquals(transfer.value, requireNotNull(unpackTransferData(route.params)).value)
    }

    @Test
    fun `an address without an amount opens the amount input`() = runTest {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)

        val route = navigation(GemPaymentTarget.Amount(asset.toGem(), recipient())).routes(payment).single()

        assertEquals(AmountParams.Transfer(asset.id, recipient()), AmountParams.unpack((route as AmountRoute).params))
    }

    @Test
    fun `an asset Core resolved opens its recipient input`() = runTest {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)

        val route = navigation(GemPaymentTarget.Recipient(asset.toGem(), recipient())).routes(payment).single()

        assertEquals(asset.id, (route as RecipientInputRoute).assetId)
    }

    @Test
    fun `several payable chains ask which asset to send`() = runTest {
        val route = navigation(GemPaymentTarget.SelectAsset(recipient(), listOf(Chain.Solana.string, Chain.Ethereum.string))).routes(payment).single()

        assertEquals(listOf(Chain.Solana, Chain.Ethereum), (route as SendSelectRoute).chains)
    }

    @Test
    fun `a payment that needs identity data opens its form`() = runTest {
        val link = PaymentLink.WalletConnectPay("pay_1")
        val route = navigation(GemPaymentTarget.Verify("https://pay.walletconnect.com/collect", link)).routes(Payment.Link(link)).single()
        assertEquals(PaymentVerificationRoute("https://pay.walletconnect.com/collect", link), route)
    }

    @Test
    fun `a payment nothing can pay opens nothing`() = runTest {
        assertTrue(navigation(GemPaymentTarget.Unsupported).routes(payment).isEmpty())
    }
}
