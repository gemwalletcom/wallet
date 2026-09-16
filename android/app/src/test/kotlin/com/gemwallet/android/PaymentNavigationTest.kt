package com.gemwallet.android

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.domains.confirm.applicationMetadata
import com.gemwallet.android.domains.confirm.asset
import uniffi.gemstone.TransactionInputType
import com.wallet.core.primitives.TransferDataOutputAction
import com.wallet.core.primitives.TransferDataOutputType
import com.gemwallet.android.testkit.mockAccount
import com.wallet.core.primitives.Chain
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockGemPaymentTransaction
import com.gemwallet.android.testkit.mockPaymentRequest
import com.gemwallet.android.ui.navigation.routes.ConfirmRoute
import com.wallet.core.primitives.ApplicationMetadataSource
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.spyk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.AlienProvider
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetsServiceInterface
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.Payment
import uniffi.gemstone.PaymentLink
import java.math.BigInteger
import com.gemwallet.android.domains.confirm.unpackTransferData

class PaymentNavigationTest {


    @Test
    fun routes_paymentLink_loadsTransactionForExistingAccount() = runTest {
        val assetInfo = mockAssetInfo(
            asset = mockAssetSolanaUSDC(),
            owner = mockAccount(chain = Chain.Solana, address = SOLANA_ADDRESS),
        )
        val getWalletAssets = mockk<GetWalletAssets>()
        val paymentService = spyk(GemPaymentService(mockk<AlienProvider>()))
        val account = requireNotNull(assetInfo.owner)
        every { getWalletAssets() } returns MutableStateFlow(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns mockGemPaymentTransaction(
            account = account,
            request = mockPaymentRequest(address = account.address, assetId = assetInfo.asset.id, memo = "payment-memo"),
        )
        val navigation = PaymentNavigation(getWalletAssets, paymentService, assetsService(assetInfo.asset))

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay("https://example.com/pay"))
        )

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(unpackTransferData(route.params))
        val assetId = transfer.asset.id
        val metadataSource = transfer.inputType.applicationMetadata?.source
        val generic = transfer.inputType as TransactionInputType.Generic
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
        val getWalletAssets = mockk<GetWalletAssets>()
        val paymentService = spyk(GemPaymentService(mockk<AlienProvider>()))
        val account = requireNotNull(assetInfo.owner)
        val recipient = SOLANA_ADDRESS
        every { getWalletAssets() } returns MutableStateFlow(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns mockGemPaymentTransaction(
            account = account,
            request = mockPaymentRequest(address = recipient, assetId = assetInfo.asset.id),
        )
        val navigation = PaymentNavigation(getWalletAssets, paymentService, assetsService(assetInfo.asset))

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay("https://example.com/pay"))
        )

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(unpackTransferData(route.params))
        val assetId = transfer.asset.id
        val generic = transfer.inputType as TransactionInputType.Generic
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
        val getWalletAssets = mockk<GetWalletAssets>()
        val paymentService = spyk(GemPaymentService(mockk<AlienProvider>()))
        val account = requireNotNull(assetInfo.owner)
        val requestedAsset = mockAssetSolanaUSDC()
        every { getWalletAssets() } returns MutableStateFlow(listOf(assetInfo))
        coEvery { paymentService.load(any(), any()) } returns mockGemPaymentTransaction(
            account = account,
            request = mockPaymentRequest(address = account.address, assetId = requestedAsset.id, memo = "payment-memo"),
        )
        val navigation = PaymentNavigation(getWalletAssets, paymentService, assetsService(requestedAsset))

        val routes = navigation.routes(
            Payment.Link(PaymentLink.SolanaPay("https://example.com/pay"))
        )

        val route = routes.single() as ConfirmRoute
        val transfer = requireNotNull(unpackTransferData(route.params))
        assertEquals(requestedAsset.id, transfer.asset.id)
        assertEquals(BigInteger("19000000"), transfer.value)
    }

    private fun assetsService(asset: Asset) = mockk<GemAssetsServiceInterface> {
        coEvery { ensureTokenAsset(asset.id.toIdentifier()) } returns asset.toGem()
    }

    private companion object {
        const val SOLANA_ADDRESS = "2kT9W3q7oXg6aPvFTN6DdK3FDZEqUigw6fmNc16YwL5n"
    }
}
