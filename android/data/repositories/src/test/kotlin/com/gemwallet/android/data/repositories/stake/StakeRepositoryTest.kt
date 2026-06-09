package com.gemwallet.android.data.repositories.stake

import com.gemwallet.android.blockchain.services.StakeService
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.data.service.store.database.entities.DbDelegationBase
import com.gemwallet.android.data.service.store.database.entities.DbDelegationValidator
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.services.gemapi.GemApiStaticClient
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.StakeValidator
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import io.mockk.runs
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class StakeRepositoryTest {
    @Test
    fun sync_keepsDelegationWhenValidatorIsMissing() = runTest {
        val walletId = WalletId("wallet")
        val assetId = mockAssetId(chain = Chain.Celestia)
        val validatorId = "celestiavaloper1missing"
        val delegation = mockDelegation(
            assetId = assetId,
            state = DelegationState.Active,
            balance = "1283",
            rewards = "35",
            validatorId = validatorId,
        ).base
        val savedValidators = mutableListOf<List<DbDelegationValidator>>()
        val savedDelegations = mutableListOf<List<DbDelegationBase>>()

        val gemApiStaticClient = mockk<GemApiStaticClient> {
            coEvery { getValidators(Chain.Celestia.string) } returns listOf(StakeValidator(validatorId, "Missing Validator"))
        }
        val stakeService = mockk<StakeService> {
            coEvery { getValidators(Chain.Celestia, any()) } returns emptyList()
            coEvery { getDelegationValidators(Chain.Celestia, any()) } returns emptyList()
            coEvery { getStakeDelegations(Chain.Celestia, any()) } returns listOf(delegation)
        }
        val stakeDao = mockk<StakeDao> {
            every { getValidators(assetId, StakeProviderType.Stake) } returns flowOf(emptyList())
            coEvery { upsertValidators(capture(savedValidators)) } just runs
            coEvery { getDelegationIds(walletId, assetId) } returns emptyList()
            coEvery { updateAndDeleteDelegations(walletId, capture(savedDelegations), emptyList()) } just runs
        }

        StakeRepository(gemApiStaticClient, stakeService, stakeDao)
            .sync(walletId, assetId, address = "address", apr = 0.0)

        val validatorRecord = savedValidators.flatten().single()
        assertEquals(validatorId, validatorRecord.validatorId)
        assertEquals("Missing Validator", validatorRecord.name)
        assertEquals(false, validatorRecord.isActive)

        val delegationRecord = savedDelegations.single().single()
        assertEquals(DelegationState.Inactive, delegationRecord.state)
        assertTrue(delegationRecord.id.contains("_inactive_"))
    }

    @Test
    fun sync_doesNotAddMissingValidatorForExistingValidator() = runTest {
        val walletId = WalletId("wallet")
        val assetId = mockAssetId(chain = Chain.Celestia)
        val validator = mockDelegationValidator(chain = Chain.Celestia, id = "validator", isActive = true)
        val delegation = mockDelegation(assetId = assetId, validatorId = validator.id).base
        val savedValidators = mutableListOf<List<DbDelegationValidator>>()

        val gemApiStaticClient = mockk<GemApiStaticClient> {
            coEvery { getValidators(Chain.Celestia.string) } returns emptyList()
        }
        val stakeService = mockk<StakeService> {
            coEvery { getValidators(Chain.Celestia, any()) } returns emptyList()
            coEvery { getDelegationValidators(Chain.Celestia, any()) } returns emptyList()
            coEvery { getStakeDelegations(Chain.Celestia, any()) } returns listOf(delegation)
        }
        val stakeDao = mockk<StakeDao> {
            every { getValidators(assetId, StakeProviderType.Stake) } returns flowOf(listOf(validator.toRecord()))
            coEvery { upsertValidators(capture(savedValidators)) } just runs
            coEvery { getDelegationIds(walletId, assetId) } returns emptyList()
            coEvery { updateAndDeleteDelegations(walletId, any(), emptyList()) } just runs
        }

        StakeRepository(gemApiStaticClient, stakeService, stakeDao)
            .sync(walletId, assetId, address = "address", apr = 0.0)

        assertTrue(savedValidators.isEmpty())
    }

    @Test
    fun pickRecommendedValidator_returnsRecommendedMatch() {
        val first = mockDelegationValidator(chain = Chain.Cosmos, id = "first")
        val recommended = mockDelegationValidator(chain = Chain.Cosmos, id = "recommended")

        val result = pickRecommendedValidator(
            validators = listOf(first, recommended),
            recommendedIds = listOf("recommended", "missing"),
        )

        assertEquals("recommended", result?.id)
    }

    @Test
    fun pickRecommendedValidator_returnsFirstValidatorWhenRecommendedMissing() {
        val first = mockDelegationValidator(chain = Chain.Bitcoin, id = "first", apr = 12.0)
        val second = mockDelegationValidator(chain = Chain.Bitcoin, id = "second")

        val result = pickRecommendedValidator(
            validators = listOf(first, second),
            recommendedIds = listOf("missing"),
        )

        assertEquals("first", result?.id)
    }

    @Test
    fun selectableValidators_filtersOutUnnamedValidators() {
        val active = mockDelegationValidator(chain = Chain.Bitcoin, id = "active", name = "Active", apr = 10.0)
        val unnamed = mockDelegationValidator(chain = Chain.Bitcoin, name = "")
        val inactive = mockDelegationValidator(chain = Chain.Bitcoin, id = "inactive", isActive = false, apr = 20.0)

        val result = selectableValidators(
            listOf(active, unnamed, inactive),
        )

        assertEquals(listOf(active.id), result.map { it.id })
    }

    @Test
    fun selectableValidators_sortsByAprDescending() {
        val lowApr = mockDelegationValidator(chain = Chain.Bitcoin, id = "low", name = "Low", apr = 4.0)
        val highApr = mockDelegationValidator(chain = Chain.Bitcoin, id = "high", name = "High", apr = 12.0)

        val result = selectableValidators(
            listOf(lowApr, highApr),
        )

        assertEquals(listOf(highApr.id, lowApr.id), result.map { it.id })
    }
}
