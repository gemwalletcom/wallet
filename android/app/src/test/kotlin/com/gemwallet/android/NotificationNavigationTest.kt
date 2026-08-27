package com.gemwallet.android

import com.gemwallet.android.application.assets.coordinators.GetAssetById
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.cases.transactions.CreateTransaction
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemTransactionsService
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Transaction
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.model.PushNotificationData
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
import com.wallet.core.primitives.PushNotificationTypes
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.Wallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test

class NotificationNavigationTest {
    private val currentWallet = mockWallet(id = "current-wallet")
    private val session = MutableStateFlow(mockSession(wallet = currentWallet))
    private val sessionRepository = mockk<SessionRepository>()
    private val walletsRepository = mockk<WalletsRepository>()
    private val createTransaction = mockk<CreateTransaction>()
    private val transactionsService = mockk<GemTransactionsService>()
    private val prefetchAssets = mockk<PrefetchAssets>()
    private val assetsService = mockk<GemAssetsService>(relaxed = true)
    private val getAssetById = mockk<GetAssetById>()

    private val subject = NotificationNavigation(
        sessionRepository = sessionRepository,
        walletsRepository = walletsRepository,
        createTransaction = createTransaction,
        transactionsService = transactionsService,
        prefetchAssets = prefetchAssets,
        assetsService = assetsService,
        getAssetById = getAssetById,
    )

    @Before
    fun setup() {
        every { sessionRepository.session() } returns session
        coEvery { sessionRepository.getCurrentCurrency() } returns Currency.USD
        coEvery { sessionRepository.setWallet(any()) } coAnswers {
            session.value = mockSession(wallet = invocation.args.first() as Wallet)
        }
        coEvery { createTransaction.createTransaction(any(), any(), any()) } answers { secondArg() }
        every { transactionsService.associatedAssetIds(any()) } answers {
            val pushed = firstArg<String>().decodeJson<Transaction>()
            listOf(pushed.assetId.toIdentifier(), pushed.feeAssetId.toIdentifier())
        }
        coEvery { prefetchAssets.prefetchAssets(any()) } returns emptyList()
        every { getAssetById(any()) } returns flowOf(null)
    }

    @Test
    fun transactionNotification_preloadsWalletDataBeforeReturningRoute() = runBlocking {
        val assetId = mockAssetId(Chain.Ethereum)
        val walletId = mockWalletId()
        val asset = mockAsset(chain = assetId.chain, tokenId = assetId.tokenId)
        val transaction = mockTransaction(assetId = assetId)
        val wallet = mockWallet(
            id = walletId.id,
            accounts = listOf(mockAccount(chain = assetId.chain)),
        )
        val assetIds = listOf(transaction.assetId, transaction.feeAssetId).distinct()
        every { walletsRepository.getWallet(wallet.id) } returns flowOf(wallet)
        every { getAssetById(assetId) } returns flowOf(asset)

        val route = subject.prepareNavigation(
            type = PushNotificationTypes.Transaction.string,
            data = PushNotificationData.Transaction(
                walletId = walletId,
                assetId = assetId,
                transaction = transaction,
            ),
        )

        assertEquals(listOf(AssetRoute(asset.id), TransactionDetailsRoute(transaction.id)), route)
        coVerify { prefetchAssets.prefetchAssets(assetIds) }
        coVerify { assetsService.addMissingBalances(wallet.id.id, any()) }
        coVerify { sessionRepository.setWallet(wallet) }
        coVerify { createTransaction.createTransaction(walletId, transaction, any()) }
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
        every { walletsRepository.getWallet(wallet.id) } returns flowOf(wallet)
        every { getAssetById(assetId) } returns flowOf(asset)

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
        verify { getAssetById(assetId) }
        coVerify { createTransaction.createTransaction(walletId, transaction, any()) }
    }

    @Test
    fun walletNotification_isRejectedWhenWalletDoesNotExist() = runBlocking {
        val assetId = mockAssetId(Chain.Solana)
        val walletId = mockWalletId("missing-wallet")
        every { walletsRepository.getWallet(walletId) } returns flowOf(null)

        val route = subject.prepareNavigation(
            type = PushNotificationTypes.Stake.string,
            data = PushNotificationData.Stake(assetId = assetId, walletId = walletId),
        )

        assertEquals(emptyList<Any>(), route)
        coVerify(exactly = 0) { prefetchAssets.prefetchAssets(any()) }
        coVerify(exactly = 0) { assetsService.addMissingBalances(any(), any()) }
        coVerify(exactly = 0) { sessionRepository.setWallet(any()) }
        coVerify(exactly = 0) { createTransaction.createTransaction(any(), any(), any()) }
    }

    @Test
    fun supportNotification_doesNotNeedPayloadData() = runBlocking {
        val route = subject.prepareNavigation(type = null, data = PushNotificationData.Support)

        assertEquals(listOf(SupportRoute), route)
        coVerify(exactly = 0) { prefetchAssets.prefetchAssets(any()) }
        coVerify(exactly = 0) { assetsService.addMissingBalances(any(), any()) }
        coVerify(exactly = 0) { sessionRepository.setWallet(any()) }
        coVerify(exactly = 0) { createTransaction.createTransaction(any(), any(), any()) }
    }

    @Test
    fun assetNotification_prefetchesAssetAndReturnsAssetRoute() = runBlocking {
        val assetId = mockAssetId(Chain.Solana)

        val route = subject.prepareNavigation(
            type = PushNotificationTypes.Asset.string,
            data = PushNotificationData.Asset(assetId),
        )

        assertEquals(listOf(AssetRoute(assetId)), route)
        coVerify { prefetchAssets.prefetchAssets(listOf(assetId)) }
        coVerify(exactly = 0) { sessionRepository.setWallet(any()) }
        coVerify(exactly = 0) { createTransaction.createTransaction(any(), any(), any()) }
    }
}
