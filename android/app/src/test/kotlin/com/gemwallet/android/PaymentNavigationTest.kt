package com.gemwallet.android

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.unpackTransferData
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockPaymentInvoice
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockTransferDataExtra
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.gemwallet.android.ui.navigation.routes.PaymentVerificationRoute
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPaymentLoad
import uniffi.gemstone.GemPaymentServiceInterface
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.Payment
import uniffi.gemstone.PaymentLink
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

class PaymentNavigationTest {

    @Test
    fun routes_paymentLink_confirmsTheLoadedTransfer() = runTest {
        val paymentService = mockk<GemPaymentServiceInterface>()
        coEvery { paymentService.load(any(), any()) } returns GemPaymentLoad.Sign(paymentTransfer())
        val navigation = PaymentNavigation(mockk(), getSession(), paymentService)

        val routes = navigation.routes(Payment.Link(PaymentLink.SolanaPay(PAYMENT_URL)))

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(unpackTransferData(route.params))
        val payment = transfer.inputType as TransactionInputType.Payment
        assertEquals(mockAssetSolana().id, transfer.asset.id)
        assertEquals(SOLANA_ADDRESS, transfer.recipient.address)
        assertEquals(BigInteger("19000000"), transfer.value)
        assertEquals(mockPaymentInvoice(link = PaymentLink.SolanaPay(PAYMENT_URL)), payment.invoice)
    }

    private fun getSession() = mockk<GetSession> {
        every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.Solana, address = SOLANA_ADDRESS)))))
    }

    @Test
    fun routes_paymentLink_opensTheVerification() = runTest {
        val paymentService = mockk<GemPaymentServiceInterface>()
        coEvery { paymentService.load(any(), any()) } returns GemPaymentLoad.Verify(
            invoice = mockPaymentInvoice(link = PaymentLink.SolanaPay(PAYMENT_URL)),
            assetId = mockAssetSolana().id.toIdentifier(),
            url = VERIFICATION_URL,
        )
        val navigation = PaymentNavigation(mockk(), getSession(), paymentService)

        val route = navigation.routes(Payment.Link(PaymentLink.SolanaPay(PAYMENT_URL))).single() as PaymentVerificationRoute

        assertEquals(VERIFICATION_URL, route.url)
        assertEquals(PaymentLink.SolanaPay(PAYMENT_URL), route.link)
    }

    private fun paymentTransfer() = mockGemTransferData(
        asset = mockAssetSolana(),
        inputType = TransactionInputType.Payment(
            asset = mockAssetSolana().toGem(),
            invoice = mockPaymentInvoice(link = PaymentLink.SolanaPay(PAYMENT_URL)),
            extra = mockTransferDataExtra(to = SOLANA_ADDRESS, data = "encoded-transaction".toByteArray()),
        ),
        recipient = GemRecipient(address = SOLANA_ADDRESS),
        value = BigInteger("19000000"),
    )

    private companion object {
        const val PAYMENT_URL = "https://example.com/pay"
        const val SOLANA_ADDRESS = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
        const val VERIFICATION_URL = "https://pay.walletconnect.com/collect/?pid=pay_1"
    }
}
