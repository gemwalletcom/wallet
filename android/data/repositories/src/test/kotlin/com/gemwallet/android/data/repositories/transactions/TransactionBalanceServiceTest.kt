package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockAssetMonad
import com.wallet.core.primitives.TransactionType
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class TransactionBalanceServiceTest {

    private val stakeRepository = mockk<StakeRepository>()
    private val perpetualRepository = mockk<PerpetualRepository>()

    private val subject = TransactionBalanceService(
        stakeRepository = stakeRepository,
        perpetualRepository = perpetualRepository,
    )

    @Test
    fun getBalance_rewards_usesRepositoryRewardsForAmountAndConfirmFlows() = runBlocking {
        val asset = mockAssetMonad()
        val assetInfo = mockAssetInfo(
            asset = asset,
            balance = AssetBalance.create(asset = asset, available = "2", rewards = "3"),
        )
        val rewards = listOf(
            mockDelegation(assetId = asset.id, balance = "2", rewards = "53"),
            mockDelegation(assetId = asset.id, balance = "100", rewards = "7"),
        )
        coEvery { stakeRepository.getRewards(asset.id, "wallet-address") } returns rewards

        val amountParams = AmountParams.buildStake(asset.id, TransactionType.StakeRewards)
        val confirmParams = ConfirmParams.Builder(
            asset = asset,
            from = requireNotNull(assetInfo.owner),
        ).rewards(
            validators = listOf(mockDelegationValidator(chain = asset.id.chain)),
        )

        assertEquals(
            BigInteger("60"),
            subject.getBalance(assetInfo, amountParams),
        )
        assertEquals(
            BigInteger("60"),
            subject.getBalance(assetInfo, confirmParams),
        )
    }

    @Test
    fun getBalance_withdraw_usesFreshDelegationBalanceFromRepository() = runBlocking {
        val asset = mockAssetCosmos()
        val assetInfo = mockAssetInfo(
            asset = asset,
            balance = AssetBalance.create(asset = asset, frozen = "0"),
        )
        val delegation = mockDelegation(
            assetId = asset.id,
            balance = "10",
            delegationId = "delegation-1",
            validatorId = "validator-1",
        )
        every {
            stakeRepository.getDelegation("validator-1", "delegation-1")
        } returns flowOf(
            mockDelegation(
                assetId = asset.id,
                balance = "44",
                delegationId = "delegation-1",
                validatorId = "validator-1",
            )
        )

        val params = ConfirmParams.Builder(
            asset = asset,
            from = requireNotNull(assetInfo.owner),
        ).withdraw(delegation)

        assertEquals(
            BigInteger("44"),
            subject.getBalance(assetInfo, params),
        )
    }

    @Test
    fun getBalance_redelegate_usesDelegationIdForFreshBalanceLookup() = runBlocking {
        val asset = mockAssetCosmos()
        val assetInfo = mockAssetInfo(
            asset = asset,
            balance = AssetBalance.create(asset = asset, frozen = "0"),
        )
        val delegation = mockDelegation(
            assetId = asset.id,
            balance = "10",
            delegationId = "delegation-1",
            validatorId = "validator-1",
        )
        every {
            stakeRepository.getDelegation("validator-1", "delegation-1")
        } returns flowOf(
            mockDelegation(
                assetId = asset.id,
                balance = "44",
                delegationId = "delegation-1",
                validatorId = "validator-1",
            )
        )

        val params = ConfirmParams.Builder(
            asset = asset,
            from = requireNotNull(assetInfo.owner),
        ).redelegate(
            destinationValidator = mockDelegationValidator(chain = asset.id.chain, id = "validator-2"),
            delegation = delegation,
        )

        assertEquals(
            BigInteger("44"),
            subject.getBalance(assetInfo, params),
        )
    }
}
