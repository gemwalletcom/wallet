package com.gemwallet.android.data.repositories.addresses

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.DbAddress
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.VerificationStatus
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class AddressesRepositoryTest {

    private val addressesDao = mockk<AddressesDao>(relaxed = true)
    private val gemDeviceApiClient = mockk<GemDeviceApiClient>(relaxed = true)
    private val repository = AddressesRepository(addressesDao, gemDeviceApiClient)

    @Test
    fun getAddressNames_usesCacheAndFetchesOnlyMissingThenSaves() = runTest {
        val cached = ChainAddress(Chain.Ethereum, "0xcached")
        val missing = ChainAddress(Chain.Ethereum, "0xmissing")
        coEvery { addressesDao.get(Chain.Ethereum, "0xcached") } returns
            DbAddress(Chain.Ethereum, "0xcached", null, "Cached", AddressType.Contact, VerificationStatus.Verified)
        coEvery { addressesDao.get(Chain.Ethereum, "0xmissing") } returns null
        coEvery { gemDeviceApiClient.getAddressNames(listOf(missing)) } returns
            listOf(AddressName(Chain.Ethereum, "0xmissing", "USDC", AddressType.Contact, VerificationStatus.Verified))

        val result = repository.getAddressNames(listOf(cached, missing))

        assertEquals(listOf("Cached", "USDC"), result.map { it.name })
        coVerify(exactly = 1) { gemDeviceApiClient.getAddressNames(listOf(missing)) }
        coVerify { addressesDao.insert(any()) }
    }
}
