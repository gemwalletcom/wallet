package com.gemwallet.android.domains.transaction

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetMonad
import com.gemwallet.android.testkit.mockAssetTron
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.Resource
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class TransactionBalanceExtTest {

    @Test
    fun testAvailableAtomicBalance() {
        val monadAsset = mockAssetMonad()
        val monadInfo = mockAssetInfo(
            asset = monadAsset,
            balance = AssetBalance.create(
                asset = monadAsset,
                available = "2",
                rewards = "53",
            ),
        )
        assertEquals(
            BigInteger("53"),
            monadInfo.balance(TransactionType.StakeRewards),
        )
        assertEquals(
            BigInteger("60"),
            monadInfo.balance(
                TransactionType.StakeRewards,
                TransactionBalanceContext(rewardsBalance = BigInteger("60")),
            ),
        )

        val cosmosAsset = mockAssetCosmos()
        val cosmosInfo = mockAssetInfo(
            asset = cosmosAsset,
            balance = AssetBalance.create(asset = cosmosAsset, available = "12"),
        )
        assertEquals(
            BigInteger("44"),
            cosmosInfo.balance(
                TransactionType.StakeUndelegate,
                TransactionBalanceContext(delegationBalance = BigInteger("44")),
            ),
        )

        val frozenInfo = mockAssetInfo(
            asset = cosmosAsset,
            balance = AssetBalance.create(asset = cosmosAsset, frozen = "99"),
        )
        assertEquals(
            BigInteger("44"),
            frozenInfo.balance(
                TransactionType.StakeWithdraw,
                TransactionBalanceContext(delegationBalance = BigInteger("44")),
            ),
        )

        val tronAsset = mockAssetTron()
        val tronInfo = mockAssetInfo(
            asset = tronAsset,
            balance = AssetBalance.create(
                asset = tronAsset,
                available = "10",
                locked = "7",
                staked = "5",
            ),
        )
        assertEquals(
            BigInteger("12"),
            tronInfo.balance(TransactionType.StakeDelegate),
        )

        assertEquals(
            BigInteger("7"),
            tronInfo.balance(
                TransactionType.StakeUnfreeze,
                TransactionBalanceContext(resource = Resource.Energy),
            ),
        )
    }
}
