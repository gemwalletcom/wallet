package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.DelegationQuery
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.math.BigInteger

@RunWith(AndroidJUnit4::class)
class DelegationQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = DelegationQuery(database.stakeDao())
    private val wallet1 = WalletId("wallet-1")
    private val wallet2 = WalletId("wallet-2")
    private val cosmos = AssetId(Chain.Cosmos)
    private val stakeValidator = mockDelegationValidator(chain = Chain.Cosmos, id = "stake")
    private val earnValidator = mockDelegationValidator(chain = Chain.Cosmos, id = "earn", providerType = StakeProviderType.Earn)
    private val own = mockDelegation(assetId = cosmos, balance = BigInteger("123456789012345678901234567890"), rewards = BigInteger("55"), delegationId = "d1", validatorId = "stake", validator = stakeValidator)
    private val ownEarn = mockDelegation(assetId = cosmos, balance = BigInteger("500"), delegationId = "d1", validatorId = "earn", validator = earnValidator)
    private val otherWallet = mockDelegation(assetId = cosmos, balance = BigInteger("999"), delegationId = "d1", validatorId = "stake", validator = stakeValidator)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        listOf(wallet1, wallet2).forEach { id ->
            database.walletsDao().insert(DbWallet(id = id.id, name = id.id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.assetsDao().insert(DbAsset(id = "cosmos", chain = Chain.Cosmos, name = "Cosmos", symbol = "ATOM", decimals = 6, type = AssetType.NATIVE))
        database.stakeDao().upsertValidators(listOf(stakeValidator, earnValidator).map { it.toRecord() })
        database.stakeDao().upsertDelegations(listOf(own.base, ownEarn.base).toRecord(wallet1) + listOf(otherWallet.base).toRecord(wallet2))
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theDelegationIsFoundByWalletValidatorAndDelegationIdWithItsExactValues() = runBlocking(Dispatchers.IO) {
        assertEquals(own, query(wallet1, "stake", "d1").first())
        assertEquals(ownEarn, query(wallet1, "earn", "d1").first())
        assertEquals(otherWallet, query(wallet2, "stake", "d1").first())
    }

    @Test
    fun aDelegationOutsideTheWalletOrValidatorIsNotFound() = runBlocking(Dispatchers.IO) {
        assertNull(query(wallet2, "earn", "d1").first())
        assertNull(query(wallet1, "stake", "d2").first())
        assertNull(query(wallet1, "cosmos_stake", "d1").first())
    }
}
