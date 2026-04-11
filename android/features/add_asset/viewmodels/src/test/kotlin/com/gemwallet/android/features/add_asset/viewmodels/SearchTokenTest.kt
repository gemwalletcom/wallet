package com.gemwallet.android.features.add_asset.viewmodels

import com.gemwallet.android.application.add_asset.coordinators.ObserveToken
import com.gemwallet.android.application.add_asset.coordinators.SearchCustomToken
import com.gemwallet.android.features.add_asset.viewmodels.models.TokenSearchState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class SearchTokenTest {

    private val searchCustomToken = mockk<SearchCustomToken>()
    private val observeToken = mockk<ObserveToken>()

    @Test
    fun `search success and token found returns Idle`() = runTest {
        val token = mockk<Asset> { every { id } returns AssetId(Chain.Ethereum, "0x123") }
        coEvery { searchCustomToken(any()) } returns true
        coEvery { observeToken(any()) } returns flowOf(token)

        val result = searchToken(searchCustomToken, observeToken, Chain.Ethereum, "0x123")

        assertEquals(TokenSearchState.Idle, result)
    }

    @Test
    fun `search returns true but token not in db returns Error`() = runTest {
        coEvery { searchCustomToken(any()) } returns true
        coEvery { observeToken(any()) } returns flowOf(null)

        val result = searchToken(searchCustomToken, observeToken, Chain.Ethereum, "0x123")

        assertEquals(TokenSearchState.Error, result)
    }

    @Test
    fun `search throws exception returns Error`() = runTest {
        coEvery { searchCustomToken(any()) } throws RuntimeException("Network error")

        val result = searchToken(searchCustomToken, observeToken, Chain.Ethereum, "0x123")

        assertEquals(TokenSearchState.Error, result)
    }
}
