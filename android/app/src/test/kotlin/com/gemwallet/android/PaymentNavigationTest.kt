package com.gemwallet.android

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.domains.confirm.applicationMetadata
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.serializer.decodeJson
import uniffi.gemstone.GemTransactionInputType
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAccount
import com.wallet.core.primitives.Chain
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
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
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import com.wallet.core.primitives.ChainAddress
import uniffi.gemstone.GemPaymentTransaction
import uniffi.gemstone.AlienProvider
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetsServiceInterface
import uniffi.gemstone.GemPaymentService
import java.math.BigInteger
import com.gemwallet.android.domains.confirm.unpackTransferData

class PaymentNavigationTest {


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
        val navigation = PaymentNavigation(getSelectAssetsInfo, paymentService, assetsService(assetInfo.asset))

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay(PaymentLinkSolanaPayInner("https://example.com/pay")))
        )

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(unpackTransferData(route.params))
        val assetId = transfer.inputType.asset.id
        val metadataSource = transfer.inputType.applicationMetadata?.source
        val generic = transfer.inputType as GemTransactionInputType.Generic
        assertEquals("encoded-transaction", String(requireNotNull(generic.extra.data)))
        assertEquals("payment-memo", transfer.recipient.memo)
        assertEquals(assetInfo.asset.id, assetId)
        assertEquals(account.address, transfer.recipient.address)
        assertEquals(BigInteger("19000000"), transfer.value)
        assertEquals(ApplicationMetadataSource.Payment, metadataSource)
        assertEquals(TransferDataOutputType.EncodedTransaction, generic.extra.outputType.toPrimitives())
        assertEquals(TransferDataOutputAction.Send, generic.extra.outputAction.toPrimitives())
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
        val navigation = PaymentNavigation(getSelectAssetsInfo, paymentService, assetsService(assetInfo.asset))

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay(PaymentLinkSolanaPayInner("https://example.com/pay")))
        )

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(unpackTransferData(route.params))
        val assetId = transfer.inputType.asset.id
        val generic = transfer.inputType as GemTransactionInputType.Generic
        assertEquals("encoded-transaction", String(requireNotNull(generic.extra.data)))
        assertEquals(null, transfer.recipient.memo)
        assertEquals(assetInfo.asset.id, assetId)
        assertEquals(recipient, transfer.recipient.address)
        assertEquals(BigInteger("19000000"), transfer.value)
    }

    @Test
    fun routes_paymentLink_asksCoreForTheRequestAssetInsteadOfTheEnabledList() = runTest {
        val assetInfo = mockAssetInfo(
            asset = mockAssetSolana(),
            owner = mockAccount(chain = Chain.Solana, address = SOLANA_ADDRESS),
        )
        val getSelectAssetsInfo = mockk<GetSelectAssetsInfo>()
        val paymentService = spyk(GemPaymentService(mockk<AlienProvider>()))
        val account = requireNotNull(assetInfo.owner)
        val requestedAsset = mockAssetSolanaUSDC()
        every { getSelectAssetsInfo() } returns flowOf(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns paymentTransaction(
            account = account,
            memo = "payment-memo",
            request = PaymentRequest(
                address = account.address,
                amount = PaymentAmount.AtomicValue("19000000"),
                memo = "payment-memo",
                references = null,
                assetId = requestedAsset.id,
            ),
        )
        val navigation = PaymentNavigation(getSelectAssetsInfo, paymentService, assetsService(requestedAsset))

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay(PaymentLinkSolanaPayInner("https://example.com/pay")))
        )

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(unpackTransferData(route.params))
        assertEquals(requestedAsset.id, transfer.inputType.asset.id)
        assertEquals(BigInteger("19000000"), transfer.value)
    }

    private fun assetsService(asset: Asset) = mockk<GemAssetsServiceInterface> {
        coEvery { ensureTokenAsset(asset.id.toIdentifier()) } returns asset.toGem()
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
        ).toGem(),
        account = ChainAddress(account.chain, account.address).toJson(),
        transaction = "encoded-transaction",
        transactionType = TransactionType.Transfer.toGem(),
        memo = memo,
        request = request?.toJson(),
    )

    private companion object {
        const val SOLANA_ADDRESS = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
    }
}
