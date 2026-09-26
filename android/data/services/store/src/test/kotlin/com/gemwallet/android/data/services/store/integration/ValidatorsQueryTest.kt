package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.ValidatorsQuery
import com.gemwallet.android.testkit.mockDelegationValidator
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ValidatorsQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = ValidatorsQuery(database.stakeDao())
    private val cosmos = AssetId(Chain.Cosmos)
    private val osmosis = AssetId(Chain.Osmosis)
    private val low = mockDelegationValidator(chain = Chain.Cosmos, id = "low", apr = 5.0)
    private val high = mockDelegationValidator(chain = Chain.Cosmos, id = "high", apr = 12.0)
    private val earn = mockDelegationValidator(chain = Chain.Cosmos, id = "earn", apr = 8.0, providerType = StakeProviderType.Earn)
    private val osmo = mockDelegationValidator(chain = Chain.Osmosis, id = "osmo", apr = 20.0)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database.assetsDao().insert(DbAsset(id = "cosmos", chain = Chain.Cosmos, name = "Cosmos", symbol = "ATOM", decimals = 6, type = AssetType.NATIVE))
        database.assetsDao().insert(DbAsset(id = "osmosis", chain = Chain.Osmosis, name = "Osmosis", symbol = "OSMO", decimals = 6, type = AssetType.NATIVE))
        database.stakeDao().upsertValidators(listOf(low, osmo, earn, high).map { it.toRecord() })
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theAssetValidatorsOfTheProviderTypeComeByAprDescending() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(high, low), query(cosmos, StakeProviderType.Stake).first())
        assertEquals(listOf(earn), query(cosmos, StakeProviderType.Earn).first())
        assertEquals(listOf(osmo), query(osmosis, StakeProviderType.Stake).first())
    }

    @Test
    fun aDeactivatedValidatorStaysListedWithoutItsApr() = runBlocking(Dispatchers.IO) {
        database.stakeDao().deactivateValidators(cosmos, listOf("high"))

        assertEquals(listOf(low, high.copy(isActive = false, apr = 0.0)), query(cosmos, StakeProviderType.Stake).first())
    }

    @Test
    fun anAssetWithoutValidatorsOfTheProviderTypeListsNothing() = runBlocking(Dispatchers.IO) {
        assertEquals(emptyList<DelegationValidator>(), query(osmosis, StakeProviderType.Earn).first())
    }
}
