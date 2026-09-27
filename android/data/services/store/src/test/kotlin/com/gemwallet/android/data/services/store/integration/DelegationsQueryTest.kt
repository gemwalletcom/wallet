package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.DelegationsQuery
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationBase
import com.gemwallet.android.testkit.mockDelegationValidator
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.math.BigInteger

@RunWith(AndroidJUnit4::class)
class DelegationsQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = DelegationsQuery(database.stakeDao())
    private val wallet1 = WalletId("wallet-1")
    private val wallet2 = WalletId("wallet-2")
    private val cosmos = AssetId(Chain.Cosmos)
    private val osmosis = AssetId(Chain.Osmosis)
    private val stakeValidator = mockDelegationValidator(chain = Chain.Cosmos, id = "stake")
    private val earnValidator = mockDelegationValidator(chain = Chain.Cosmos, id = "earn", providerType = StakeProviderType.Earn)
    private val osmoValidator = mockDelegationValidator(chain = Chain.Osmosis, id = "osmo")
    private val staked =
        mockDelegation(
            base = mockDelegationBase(
                assetId = cosmos,
                balance = BigInteger("123456789012345678901234567890"),
                shares = BigInteger("123456789012345678901234567890"),
                rewards = BigInteger("987654321"),
                delegationId = "d1",
                validatorId = "stake",
            ),
            validator = stakeValidator,
        )
    private val earning = mockDelegation(base = mockDelegationBase(assetId = cosmos, balance = BigInteger("500"), shares = BigInteger("500"), delegationId = "d2", validatorId = "earn"), validator = earnValidator)
    private val otherWallet = mockDelegation(base = mockDelegationBase(assetId = cosmos, balance = BigInteger("999"), shares = BigInteger("999"), delegationId = "d3", validatorId = "stake"), validator = stakeValidator)
    private val osmo = mockDelegation(base = mockDelegationBase(assetId = osmosis, balance = BigInteger("42"), shares = BigInteger("42"), delegationId = "d4", validatorId = "osmo"), validator = osmoValidator)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        listOf(wallet1, wallet2).forEach { id ->
            database.walletsDao().insert(DbWallet(id = id.id, name = id.id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.assetsDao().insert(DbAsset(id = "cosmos", chain = Chain.Cosmos, name = "Cosmos", symbol = "ATOM", decimals = 6, type = AssetType.NATIVE))
        database.assetsDao().insert(DbAsset(id = "osmosis", chain = Chain.Osmosis, name = "Osmosis", symbol = "OSMO", decimals = 6, type = AssetType.NATIVE))
        database.stakeDao().upsertValidators(listOf(stakeValidator, earnValidator, osmoValidator).map { it.toRecord() })
        database.stakeDao().upsertDelegations(listOf(staked.base, earning.base, osmo.base).toRecord(wallet1) + listOf(otherWallet.base).toRecord(wallet2))
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theWalletDelegationsOfTheAssetAndProviderTypeKeepTheirExactValues() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(staked), query(wallet1, cosmos, StakeProviderType.Stake).first())
        assertEquals(listOf(earning), query(wallet1, cosmos, StakeProviderType.Earn).first())
        assertEquals(listOf(osmo), query(wallet1, osmosis, StakeProviderType.Stake).first())
    }

    @Test
    fun anotherWalletSeesOnlyItsOwnDelegations() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(otherWallet), query(wallet2, cosmos, StakeProviderType.Stake).first())
        assertEquals(emptyList<Delegation>(), query(wallet2, osmosis, StakeProviderType.Stake).first())
    }

    @Test
    fun severalDelegationsOfTheWalletAreAllListed() = runBlocking(Dispatchers.IO) {
        val second = mockDelegation(base = mockDelegationBase(assetId = cosmos, balance = BigInteger("7"), shares = BigInteger("7"), delegationId = "d5", validatorId = "stake"), validator = stakeValidator)
        database.stakeDao().upsertDelegations(listOf(second.base).toRecord(wallet1))

        assertEquals(setOf(staked, second), query(wallet1, cosmos, StakeProviderType.Stake).first().toSet())
    }
}
