package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockAssetMonad
import com.gemwallet.android.testkit.mockWalletId
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class TransactionBalanceServiceTest {

    private val perpetualRepository = mockk<PerpetualRepository>()
    private val sessionRepository = mockk<SessionRepository>(relaxed = true)

    private val subject = TransactionBalanceService(
        perpetualRepository = perpetualRepository,
        sessionRepository = sessionRepository,
    )

    @Test
    fun getBalance_rewards_usesTheRewardsAmount() = runBlocking {
        val asset = mockAssetMonad()
        val assetInfo = mockAssetInfo(
            asset = asset,
            balance = AssetBalance.create(asset = asset, available = "2", rewards = "3"),
        )

        val confirmParams = ConfirmParams.Builder(
            asset = asset,
            from = requireNotNull(assetInfo.owner),
            amount = BigInteger("60"),
        ).rewards(
            validators = listOf(mockDelegationValidator(chain = asset.id.chain)),
        )

        assertEquals(
            BigInteger("60"),
            subject.getBalance(assetInfo, confirmParams),
        )
    }

    @Test
    fun getBalance_withdrawAndRedelegate_useDelegationBalance() = runBlocking {
        val asset = mockAssetCosmos()
        val assetInfo = mockAssetInfo(
            asset = asset,
            balance = AssetBalance.create(asset = asset, available = "1"),
        )
        val delegation = mockDelegation(assetId = asset.id, balance = "10", delegationId = "delegation-1", validatorId = "validator-1")
        val from = requireNotNull(assetInfo.owner)

        assertEquals(BigInteger("10"), subject.getBalance(assetInfo, ConfirmParams.Builder(asset = asset, from = from).withdraw(delegation)))
        assertEquals(
            BigInteger("10"),
            subject.getBalance(assetInfo, ConfirmParams.Builder(asset = asset, from = from).redelegate(mockDelegationValidator(chain = asset.id.chain), delegation)),
        )
    }

}
