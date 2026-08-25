package com.gemwallet.android

import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.isValidAddress
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentLinkSolanaPayInner
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.ChainAddress
import uniffi.gemstone.GemApplicationMetadata
import uniffi.gemstone.GemApplicationMetadataSource
import uniffi.gemstone.GemPaymentAmount
import uniffi.gemstone.GemPaymentRequest
import uniffi.gemstone.GemPaymentTransaction
import uniffi.gemstone.PaymentServiceInterface
import uniffi.gemstone.TransactionType

class PaymentNavigationTest {

    @After
    fun tearDown() {
        unmockkStatic("com.gemwallet.android.ext.ChainKt")
    }

    @Test
    fun routes_paymentLink_loadsTransactionForExistingAccount() = runTest {
        val assetInfo = mockAssetInfo(asset = mockAssetSolanaUSDC())
        val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>()
        val paymentService = mockk<PaymentServiceInterface>()
        val account = requireNotNull(assetInfo.owner)
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { account.chain.checksumAddress(any()) } answers { secondArg() }
        every { account.chain.isValidAddress(any()) } returns true
        every { account.chain.isMemoSupport() } returns false
        every { getSelectAssetsInfo() } returns flowOf(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns GemPaymentTransaction(
            merchant = GemApplicationMetadata(
                name = "Merchant",
                description = "Payment",
                url = "https://example.com",
                icon = "https://example.com/icon.png",
                source = GemApplicationMetadataSource.PAYMENT,
            ),
            account = ChainAddress(account.chain.string, account.address),
            transaction = "encoded-transaction",
            transactionType = TransactionType.TRANSFER,
            memo = "payment-memo",
            request = GemPaymentRequest(
                address = account.address,
                amount = GemPaymentAmount.AtomicValue("19000000"),
                memo = "payment-memo",
                references = null,
                assetId = assetInfo.asset.id.toIdentifier(),
            ),
        )
        val navigation = PaymentNavigation(getSelectAssetsInfo, paymentService)

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay(PaymentLinkSolanaPayInner("https://example.com/pay")))
        )

        val route = routes.single() as ConfirmRoute
        val params = ConfirmParams.unpack(route.params) as ConfirmParams.TransferParams.Generic
        assertEquals("encoded-transaction", params.data)
        assertEquals("payment-memo", params.memo)
        assertEquals(assetInfo.asset.id, params.asset.id)
        assertEquals(account.address, params.destination.address)
        assertEquals("19000000", params.amount.toString())
        assertEquals(ApplicationMetadataSource.Payment, params.metadata.source)
        assertEquals(ConfirmParams.TransferParams.InputType.EncodeTransaction, params.inputType)
        assertTrue(params.isSendable)
    }
}
