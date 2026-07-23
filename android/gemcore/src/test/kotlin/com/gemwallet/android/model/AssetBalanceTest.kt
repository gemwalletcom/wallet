package com.gemwallet.android.model

import com.gemwallet.android.testkit.mockAssetTron
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class AssetBalanceTest {

    @Test
    fun testGetFrozenResourceAmount() {
        val asset = mockAssetTron()

        assertEquals(
            BigInteger("8"),
            AssetBalance.create(asset = asset, frozen = "5", locked = "3").balance.getFrozenResourceAmount(),
        )
        assertEquals(
            BigInteger("5"),
            AssetBalance.create(asset = asset, frozen = "5").balance.getFrozenResourceAmount(),
        )
        assertEquals(
            BigInteger("3"),
            AssetBalance.create(asset = asset, locked = "3").balance.getFrozenResourceAmount(),
        )
        assertEquals(
            BigInteger.ZERO,
            AssetBalance.create(asset = asset).balance.getFrozenResourceAmount(),
        )
        assertEquals(
            BigInteger.ZERO,
            AssetBalance.create(
                asset = asset,
                available = "9",
                staked = "9",
                pending = "9",
                rewards = "9",
            ).balance.getFrozenResourceAmount(),
        )
    }
}
