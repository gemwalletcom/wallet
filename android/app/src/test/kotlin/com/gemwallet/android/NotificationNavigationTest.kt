package com.gemwallet.android

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.ext.toIdentifier
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
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemPushNotification
import uniffi.gemstone.GemPushNotificationService
import com.wallet.core.primitives.WalletId

class NotificationNavigationTest {
    private val currentWallet = mockWallet(id = "current-wallet")
    private val session = MutableStateFlow(mockSession(wallet = currentWallet))
    private val getSession = mockk<GetSession>()
    private val setCurrentWallet = mockk<SetCurrentWallet>(relaxed = true)
    private val getWallet = mockk<GetWallet>()
    private val createTransaction = mockk<CreateTransaction>()
    private val assetsService = mockk<GemAssetsService>()
    private val pushNotificationService = GemPushNotificationService()

    private val subject = NotificationNavigation(
        getSession = getSession,
        setCurrentWallet = setCurrentWallet,
        getWallet = getWallet,
        createTransaction = createTransaction,
        assetsService = assetsService,
        pushNotificationService = pushNotificationService,
    )

    @Before
    fun setup() {
        every { getSession() } returns session
        coEvery { setCurrentWallet.setCurrentWallet(any()) } coAnswers {
            session.value = mockSession(wallet = mockWallet(id = (invocation.args.first() as WalletId).id))
        }
        coEvery { assetsService.syncMissingAssets(any()) } returns emptyList()
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
            GemPushNotification.Transaction(
                walletId = walletId.id,
                assetId = assetId.toIdentifier(),
                transaction = transaction.toJson(),
            )
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
            GemPushNotification.Transaction(
                walletId = walletId.id,
                assetId = assetId.toIdentifier(),
                transaction = transaction.toJson(),
            )
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
            GemPushNotification.Transaction(
                walletId = walletId.id,
                assetId = assetId.toIdentifier(),
                transaction = transaction.toJson(),
            )
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
        coEvery { assetsService.openWalletAsset(wallet.toJson(), assetId.toIdentifier()) } returns asset.toGem()

        val route = subject.prepareNavigation(
            GemPushNotification.Stake(walletId = walletId.id, assetId = assetId.toIdentifier())
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
            GemPushNotification.Stake(walletId = walletId.id, assetId = assetId.toIdentifier())
        )

        assertEquals(emptyList<Any>(), route)
        coVerify(exactly = 0) { assetsService.openWalletAsset(any(), any()) }
        coVerify(exactly = 0) { setCurrentWallet.setCurrentWallet(any()) }
        coVerify(exactly = 0) { createTransaction.createNotificationTransaction(any(), any(), any()) }
    }

    @Test
    fun supportNotification_doesNotNeedPayloadData() = runBlocking {
        val route = subject.prepareNavigation(GemPushNotification.Support)

        assertEquals(listOf(SupportRoute), route)
        coVerify(exactly = 0) { assetsService.syncMissingAssets(any()) }
        coVerify(exactly = 0) { setCurrentWallet.setCurrentWallet(any()) }
    }

    @Test
    fun assetNotification_opensAssetThroughCore() = runBlocking {
        val asset = mockAsset(chain = Chain.Solana)
        val callingThreads = mutableListOf<String>()
        coEvery { assetsService.openAsset(asset.id.toIdentifier()) } answers {
            callingThreads += Thread.currentThread().name
            asset.toGem()
        }

        val route = subject.prepareNavigation(GemPushNotification.Asset(asset.id.toIdentifier()))

        assertEquals(listOf(AssetRoute(asset.id)), route)
        coVerify(exactly = 0) { setCurrentWallet.setCurrentWallet(any()) }
        assertTrue(
            "openAsset reads the current wallet through a synchronous store callback that blocks on Room; " +
                "the notification routes are built on the main scope. Got $callingThreads",
            callingThreads.single().startsWith("DefaultDispatcher-worker"),
        )
    }

    @Test
    fun priceAlertNotification_isRejectedWhenCoreDoesNotOpenTheAsset() = runBlocking {
        val assetId = mockAssetId(Chain.Bitcoin)
        coEvery { assetsService.openAsset(assetId.toIdentifier()) } returns null

        val route = subject.prepareNavigation(GemPushNotification.PriceAlert(assetId.toIdentifier()))

        assertEquals(emptyList<Any>(), route)
    }

    @Test
    fun rewardsNotification_opensReferralWithoutPayloadData() = runBlocking {
        val notification = pushNotificationService.parse(PushNotificationTypes.Rewards.string, null)

        assertEquals(GemPushNotification.Rewards, notification)
        assertEquals(listOf(ReferralRoute()), subject.prepareNavigation(notification!!))
    }

    @Test
    fun testNotification_navigatesNowhere() = runBlocking {
        val notification = pushNotificationService.parse(PushNotificationTypes.Test.string, null)

        assertEquals(GemPushNotification.Test, notification)
        assertEquals(emptyList<Any>(), subject.prepareNavigation(notification!!))
    }
}
