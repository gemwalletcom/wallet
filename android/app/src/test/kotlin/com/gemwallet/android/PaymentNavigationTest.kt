package com.gemwallet.android

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.domains.confirm.unpackTransferData
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.spyk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemPaymentLoad
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.Payment
import uniffi.gemstone.PaymentInvoice
import uniffi.gemstone.PaymentLink
import uniffi.gemstone.PaymentMerchant
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.TransferDataExtra
import java.math.BigInteger

class PaymentNavigationTest {

    @Test
    fun routes_paymentLink_confirmsTheLoadedTransfer() = runTest {
        val assetInfo = mockAssetInfo(
            asset = mockAssetSolana(),
            owner = mockAccount(chain = Chain.Solana, address = SOLANA_ADDRESS),
        )
        val getWalletAssets = mockk<GetWalletAssets>()
        val paymentService = spyk(GemPaymentService(mockk<AlienProvider>(), mockk<GemAssetsService>()))
        every { getWalletAssets() } returns MutableStateFlow(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns GemPaymentLoad.Sign(paymentTransfer())
        val navigation = PaymentNavigation(getWalletAssets, paymentService)

        val routes = navigation.routes(Payment.Link(PaymentLink.SolanaPay(PAYMENT_URL)))

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(unpackTransferData(route.params))
        val payment = transfer.inputType as TransactionInputType.Payment
        assertEquals(mockAssetSolana().id, transfer.asset.id)
        assertEquals(SOLANA_ADDRESS, transfer.recipient.address)
        assertEquals(BigInteger("19000000"), transfer.value)
        assertEquals(paymentInvoice(), payment.invoice)
    }

    @Test
    fun routes_paymentLink_verificationHasNoRoute() = runTest {
        val getWalletAssets = mockk<GetWalletAssets>()
        val paymentService = spyk(GemPaymentService(mockk<AlienProvider>(), mockk<GemAssetsService>()))
        every { getWalletAssets() } returns MutableStateFlow(listOf(mockAssetInfo(asset = mockAssetSolana())))
        coEvery { paymentService.load(any(), any()) } returns GemPaymentLoad.Verify(
            invoice = paymentInvoice(),
            assetId = mockAssetSolana().id.toGem(),
            url = "https://walletconnect.com/verify",
        )
        val navigation = PaymentNavigation(getWalletAssets, paymentService)

        assertTrue(navigation.routes(Payment.Link(PaymentLink.SolanaPay(PAYMENT_URL))).isEmpty())
    }

    private fun paymentTransfer() = GemTransferData(
        inputType = TransactionInputType.Payment(
            asset = mockAssetSolana().toGem(),
            invoice = paymentInvoice(),
            extra = TransferDataExtra(
                to = SOLANA_ADDRESS,
                gasLimit = null,
                gasPrice = null,
                data = "encoded-transaction".toByteArray(),
                outputType = TransferDataOutputType.EncodedTransaction.toGem(),
                outputAction = TransferDataOutputAction.Send.toGem(),
                transactionType = TransactionType.Transfer.toGem(),
                approval = null,
            ),
        ),
        recipient = GemRecipient(address = SOLANA_ADDRESS, name = null, memo = null, references = emptyList()),
        value = BigInteger("19000000"),
        useMaxAmount = false,
    )

    private fun paymentInvoice() = PaymentInvoice(
        link = PaymentLink.SolanaPay(PAYMENT_URL),
        merchant = PaymentMerchant(name = "Merchant", icon = "https://example.com/icon.png"),
        price = null,
        quotes = emptyList(),
    )

    private companion object {
        const val PAYMENT_URL = "https://example.com/pay"
        const val SOLANA_ADDRESS = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
    }
}
