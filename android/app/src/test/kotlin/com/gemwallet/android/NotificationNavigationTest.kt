package com.gemwallet.android

import com.gemwallet.android.application.assets.cases.SyncMissingAssets
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.PushNotificationData
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualPositionRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.TransactionDetailsRoute
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PushNotificationTypes
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.Wallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetsService
import com.wallet.core.primitives.WalletId

class NotificationNavigationTest {
    private val currentWallet = mockWallet(id = "current-wallet")
    private val session = MutableStateFlow(mockSession(wallet = currentWallet))
    private val getSession = mockk<GetSession>()
    private val setCurrentWallet = mockk<SetCurrentWallet>(relaxed = true)
    private val getWallet = mockk<GetWallet>()
    private val createTransaction = mockk<CreateTransaction>()
    private val syncMissingAssets = mockk<SyncMissingAssets>()
    private val assetsService = mockk<GemAssetsService>()

    private val subject = NotificationNavigation(
        getSession = getSession,
        setCurrentWallet = setCurrentWallet,
        getWallet = getWallet,
        createTransaction = createTransaction,
        syncMissingAssets = syncMissingAssets,
        assetsService = assetsService,
    )

    @Before
    fun setup() {
        every { getSession() } returns session
        coEvery { setCurrentWallet.setCurrentWallet(any()) } coAnswers {
            session.value = mockSession(wallet = mockWallet(id = (invocation.args.first() as WalletId).id))
        }
        coEvery { syncMissingAssets.syncMissingAssets(any()) } returns emptyList()
    }

    @Test
    fun transactionNotification_addsTransactionThroughCoreBeforeReturningRoute() = runBlocking {
        val assetId = mockAssetId(Chain.Ethereum)
        val walletId = mockWalletId()
        val asset = mockAsset(chain = assetId.chain, tokenId = assetId.tokenId)
        val transaction = mockTransaction(assetId = assetId)
        val wallet = mockWallet(
            id = walletId.id,
            accounts = listOf(mockAccount(chain = assetId.chain)),
        )
        every { getWallet(wallet.id) } returns flowOf(wallet)
        coEvery { createTransaction.createNotificationTransaction(wallet, assetId, transaction) } returns asset

        val route = subject.prepareNavigation(
            type = PushNotificationTypes.Transaction.string,
            data = PushNotificationData.Transaction(
                walletId = walletId,
                assetId = assetId,
                transaction = transaction,
            ),
        )

        assertEquals(listOf(AssetRoute(asset.id), TransactionDetailsRoute(transaction.id)), route)
        coVerify { setCurrentWallet.setCurrentWallet(wallet.id) }
    }

    @Test
    fun transactionNotification_isRejectedWhenCoreDoesNotOpenAsset() = runBlocking {
        val assetId = mockAssetId(Chain.Ethereum)
        val walletId = mockWalletId()
        val transaction = mockTransaction(assetId = assetId)
        val wallet = mockWallet(id = walletId.id)
        every { getWallet(wallet.id) } returns flowOf(wallet)
        coEvery { createTransaction.createNotificationTransaction(wallet, assetId, transaction) } returns null

        val route = subject.prepareNavigation(
            type = PushNotificationTypes.Transaction.string,
            data = PushNotificationData.Transaction(
                walletId = walletId,
                assetId = assetId,
                transaction = transaction,
            ),
        )

        assertEquals(emptyList<Any>(), route)
        coVerify(exactly = 0) { setCurrentWallet.setCurrentWallet(any()) }
    }

    @Test
    fun perpetualTransactionNotification_opensPerpetualMarketBeforeTransaction() = runBlocking {
        val assetId = mockAssetId(Chain.HyperCore, tokenId = "perpetual::UNI")
        val walletId = mockWalletId()
        val asset = mockAsset(
            chain = assetId.chain,
            tokenId = assetId.tokenId,
            name = "Uniswap",
            symbol = "UNI",
            type = AssetType.PERPETUAL,
        )
        val transaction = mockTransaction(
            assetId = assetId,
            type = TransactionType.PerpetualOpenPosition,
        )
        val wallet = mockWallet(
            id = walletId.id,
            accounts = listOf(mockAccount(chain = assetId.chain)),
        )
        every { getWallet(wallet.id) } returns flowOf(wallet)
        coEvery { createTransaction.createNotificationTransaction(wallet, assetId, transaction) } returns asset

        val route = subject.prepareNavigation(
            type = PushNotificationTypes.Transaction.string,
            data = PushNotificationData.Transaction(
                walletId = walletId,
                assetId = assetId,
                transaction = transaction,
            ),
        )

        assertEquals(
            listOf(
                PerpetualRoute,
                PerpetualPositionRoute(assetId),
                TransactionDetailsRoute(transaction.id),
            ),
            route,
        )
    }

    @Test
    fun stakeNotification_opensWalletAssetThroughCore() = runBlocking {
        val assetId = mockAssetId(Chain.Solana)
        val walletId = mockWalletId()
        val asset = mockAsset(chain = assetId.chain, tokenId = assetId.tokenId)
        val wallet = mockWallet(
            id = walletId.id,
            accounts = listOf(mockAccount(chain = assetId.chain)),
        )
        every { getWallet(wallet.id) } returns flowOf(wallet)
        coEvery { assetsService.openWalletAsset(wallet.toJson(), assetId.toIdentifier()) } returns asset.toJson()

        val route = subject.prepareNavigation(
            type = PushNotificationTypes.Stake.string,
            data = PushNotificationData.Stake(assetId = assetId, walletId = walletId),
        )

        assertEquals(listOf(AssetRoute(asset.id)), route)
        coVerify { setCurrentWallet.setCurrentWallet(wallet.id) }
    }

    @Test
    fun walletNotification_isRejectedWhenWalletDoesNotExist() = runBlocking {
        val assetId = mockAssetId(Chain.Solana)
        val walletId = mockWalletId("missing-wallet")
        every { getWallet(walletId) } returns flowOf(null)

        val route = subject.prepareNavigation(
            type = PushNotificationTypes.Stake.string,
            data = PushNotificationData.Stake(assetId = assetId, walletId = walletId),
        )

        assertEquals(emptyList<Any>(), route)
        coVerify(exactly = 0) { assetsService.openWalletAsset(any(), any()) }
        coVerify(exactly = 0) { setCurrentWallet.setCurrentWallet(any()) }
        coVerify(exactly = 0) { createTransaction.createNotificationTransaction(any(), any(), any()) }
    }

    @Test
    fun supportNotification_doesNotNeedPayloadData() = runBlocking {
        val route = subject.prepareNavigation(type = null, data = PushNotificationData.Support)

        assertEquals(listOf(SupportRoute), route)
        coVerify(exactly = 0) { syncMissingAssets.syncMissingAssets(any()) }
        coVerify(exactly = 0) { setCurrentWallet.setCurrentWallet(any()) }
    }

    @Test
    fun assetNotification_prefetchesAssetAndReturnsAssetRoute() = runBlocking {
        val assetId = mockAssetId(Chain.Solana)

        val route = subject.prepareNavigation(
            type = PushNotificationTypes.Asset.string,
            data = PushNotificationData.Asset(assetId),
        )

        assertEquals(listOf(AssetRoute(assetId)), route)
        coVerify { syncMissingAssets.syncMissingAssets(listOf(assetId)) }
        coVerify(exactly = 0) { setCurrentWallet.setCurrentWallet(any()) }
    }
}
