package com.gemwallet.android

import uniffi.gemstone.GemTransferService
import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.isValidAddress
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.domains.confirm.applicationMetadata
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.unpack
import com.gemwallet.android.serializer.decodeJson
import uniffi.gemstone.GemTransactionInputType
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAccount
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
import io.mockk.spyk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemPaymentTransaction
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemPaymentService
import java.math.BigInteger

class PaymentNavigationTest {

    private val transferService = GemTransferService()

    @After
    fun tearDown() = unmockkStatic("com.gemwallet.android.ext.ChainKt")

    @Test
    fun routes_paymentLink_loadsTransactionForExistingAccount() = runTest {
        val assetInfo = mockAssetInfo(
            asset = mockAssetSolanaUSDC(),
            owner = mockAccount(chain = Chain.Solana, address = SOLANA_ADDRESS),
        )
        val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>()
        val paymentService = spyk(GemPaymentService(mockk<AlienProvider>()))
        val account = requireNotNull(assetInfo.owner)
        val request = PaymentRequest(
            address = account.address,
            amount = PaymentAmount.AtomicValue("19000000"),
            memo = "payment-memo",
            references = null,
            assetId = assetInfo.asset.id,
        )
        every { getSelectAssetsInfo() } returns flowOf(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns paymentTransaction(
            account = account,
            memo = "payment-memo",
            request = request,
        )
        val navigation = PaymentNavigation(getSelectAssetsInfo, paymentService, GemTransferService())

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay(PaymentLinkSolanaPayInner("https://example.com/pay")))
        )

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(transferService.unpack(route.params))
        val assetId = transfer.inputType.asset.id
        val metadataSource = transfer.inputType.applicationMetadata?.source
        val generic = transfer.inputType as GemTransactionInputType.Generic
        assertEquals("encoded-transaction", String(requireNotNull(generic.extra.data)))
        assertEquals("payment-memo", transfer.recipient.memo)
        assertEquals(assetInfo.asset.id, assetId)
        assertEquals(account.address, transfer.recipient.address)
        assertEquals(BigInteger("19000000"), transfer.value)
        assertEquals(ApplicationMetadataSource.Payment, metadataSource)
        assertEquals(TransferDataOutputType.EncodedTransaction, generic.extra.outputType.decodeJson<TransferDataOutputType>())
        assertEquals(TransferDataOutputAction.Send, generic.extra.outputAction.decodeJson<TransferDataOutputAction>())
    }

    @Test
    fun routes_paymentLink_confirmsDecodedTransferWithoutMemo() = runTest {
        val assetInfo = mockAssetInfo(asset = mockAssetSolanaUSDC())
        val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>()
        val paymentService = spyk(GemPaymentService(mockk<AlienProvider>()))
        val account = requireNotNull(assetInfo.owner)
        val recipient = SOLANA_ADDRESS
        val request = PaymentRequest(
            address = recipient,
            amount = PaymentAmount.AtomicValue("19000000"),
            memo = null,
            references = null,
            assetId = assetInfo.asset.id,
        )
        every { getSelectAssetsInfo() } returns flowOf(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns paymentTransaction(
            account = account,
            memo = null,
            request = request,
        )
        val navigation = PaymentNavigation(getSelectAssetsInfo, paymentService, GemTransferService())

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay(PaymentLinkSolanaPayInner("https://example.com/pay")))
        )

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(transferService.unpack(route.params))
        val assetId = transfer.inputType.asset.id
        val generic = transfer.inputType as GemTransactionInputType.Generic
        assertEquals("encoded-transaction", String(requireNotNull(generic.extra.data)))
        assertEquals(null, transfer.recipient.memo)
        assertEquals(assetInfo.asset.id, assetId)
        assertEquals(recipient, transfer.recipient.address)
        assertEquals(BigInteger("19000000"), transfer.value)
    }

    @Test
    fun routes_paymentLink_fallsBackToEncodedTransactionForUnknownAsset() = runTest {
        val assetInfo = mockAssetInfo(asset = mockAssetSolanaUSDC())
        val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>()
        val paymentService = spyk(GemPaymentService(mockk<AlienProvider>()))
        val account = requireNotNull(assetInfo.owner)
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { account.chain.checksumAddress(any(), any()) } answers { secondArg() }
        every { account.chain.isValidAddress(any(), any()) } returns true
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
        val navigation = PaymentNavigation(getSelectAssetsInfo, paymentService, GemTransferService())

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay(PaymentLinkSolanaPayInner("https://example.com/pay")))
        )

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(transferService.unpack(route.params))
        val assetId = transfer.inputType.asset.id
        val generic = transfer.inputType as GemTransactionInputType.Generic
        assertEquals("encoded-transaction", String(requireNotNull(generic.extra.data)))
        assertEquals("payment-memo", transfer.recipient.memo)
        assertEquals(account.chain, assetId.chain)
        assertEquals(null, assetId.tokenId)
        assertEquals("", transfer.recipient.address)
        assertEquals(BigInteger.ZERO, transfer.value)
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
        account = ChainAddress(account.chain, account.address).toJson(),
        transaction = "encoded-transaction",
        transactionType = TransactionType.Transfer.toJson(),
        memo = memo,
        request = request?.toJson(),
    )

    private companion object {
        const val SOLANA_ADDRESS = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
    }
}
