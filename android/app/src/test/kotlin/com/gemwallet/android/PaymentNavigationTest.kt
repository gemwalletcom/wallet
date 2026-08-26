package com.gemwallet.android

import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.isValidAddress
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentLinkSolanaPayInner
import com.wallet.core.primitives.PaymentRequest
import com.wallet.core.primitives.TransactionType
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
import uniffi.gemstone.GemPaymentConfirmTransfer
import uniffi.gemstone.GemPaymentTransaction
import uniffi.gemstone.PaymentServiceInterface
import uniffi.gemstone.paymentDecodedTransfer

class PaymentNavigationTest {

    @After
    fun tearDown() {
        unmockkStatic("com.gemwallet.android.ext.ChainKt")
        unmockkStatic("uniffi.gemstone.GemstoneKt")
    }

    @Test
    fun routes_paymentLink_loadsTransactionForExistingAccount() = runTest {
        val assetInfo = mockAssetInfo(asset = mockAssetSolanaUSDC())
        val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>()
        val paymentService = mockk<PaymentServiceInterface>()
        val account = requireNotNull(assetInfo.owner)
        val request = PaymentRequest(
            address = account.address,
            amount = PaymentAmount.AtomicValue("19000000"),
            memo = "payment-memo",
            references = null,
            assetId = assetInfo.asset.id,
        )
        mockkStatic("uniffi.gemstone.GemstoneKt")
        every { paymentDecodedTransfer(request.toJson(), any()) } returns GemPaymentConfirmTransfer(
            assetId = assetInfo.asset.id.toIdentifier(),
            address = account.address,
            value = "19000000",
            memo = "payment-memo",
            references = listOf(),
        )
        every { getSelectAssetsInfo() } returns flowOf(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns paymentTransaction(
            account = account,
            memo = "payment-memo",
            request = request,
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

    @Test
    fun routes_paymentLink_confirmsDecodedTransferWithoutMemo() = runTest {
        val assetInfo = mockAssetInfo(asset = mockAssetSolanaUSDC())
        val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>()
        val paymentService = mockk<PaymentServiceInterface>()
        val account = requireNotNull(assetInfo.owner)
        val recipient = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
        val request = PaymentRequest(
            address = recipient,
            amount = PaymentAmount.AtomicValue("19000000"),
            memo = null,
            references = null,
            assetId = assetInfo.asset.id,
        )
        mockkStatic("uniffi.gemstone.GemstoneKt")
        every { paymentDecodedTransfer(request.toJson(), any()) } returns GemPaymentConfirmTransfer(
            assetId = assetInfo.asset.id.toIdentifier(),
            address = recipient,
            value = "19000000",
            memo = null,
            references = listOf(),
        )
        every { getSelectAssetsInfo() } returns flowOf(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns paymentTransaction(
            account = account,
            memo = null,
            request = request,
        )
        val navigation = PaymentNavigation(getSelectAssetsInfo, paymentService)

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay(PaymentLinkSolanaPayInner("https://example.com/pay")))
        )

        val route = routes.single() as ConfirmRoute
        val params = ConfirmParams.unpack(route.params) as ConfirmParams.TransferParams.Generic
        assertEquals("encoded-transaction", params.data)
        assertEquals(null, params.memo)
        assertEquals(assetInfo.asset.id, params.asset.id)
        assertEquals(recipient, params.destination.address)
        assertEquals("19000000", params.amount.toString())
    }

    @Test
    fun routes_paymentLink_fallsBackToEncodedTransactionForUnknownAsset() = runTest {
        val assetInfo = mockAssetInfo(asset = mockAssetSolanaUSDC())
        val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>()
        val paymentService = mockk<PaymentServiceInterface>()
        val account = requireNotNull(assetInfo.owner)
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { account.chain.checksumAddress(any()) } answers { secondArg() }
        every { account.chain.isValidAddress(any()) } returns true
        every { account.chain.isMemoSupport() } returns true
        every { account.chain.asset() } returns mockAssetSolana()
        every { getSelectAssetsInfo() } returns flowOf(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns paymentTransaction(
            account = account,
            memo = "payment-memo",
            request = PaymentRequest(
                address = account.address,
                amount = PaymentAmount.AtomicValue("19000000"),
                memo = "payment-memo",
                references = null,
                assetId = AssetId(Chain.Solana),
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
        assertEquals(account.chain, params.asset.id.chain)
        assertEquals(null, params.asset.id.tokenId)
        assertEquals("", params.destination.address)
        assertEquals("0", params.amount.toString())
    }

    private fun paymentTransaction(
        account: Account,
        memo: String?,
        request: PaymentRequest?,
    ) = GemPaymentTransaction(
        merchant = ApplicationMetadata(
            name = "Merchant",
            description = "Payment",
            url = "https://example.com",
            icon = "https://example.com/icon.png",
            source = ApplicationMetadataSource.Payment,
        ).toJson(),
        account = ChainAddress(account.chain.string, account.address),
        transaction = "encoded-transaction",
        transactionType = TransactionType.Transfer.toJson(),
        memo = memo,
        request = request?.toJson(),
    )
}
