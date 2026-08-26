package com.gemwallet.android.data.repositories.addresses

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.entities.DbAddress
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
import uniffi.gemstone.GemNameService
import com.gemwallet.android.serializer.toJson
import org.junit.Test

class AddressesRepositoryTest {

    private val addressesDao = mockk<AddressesDao>(relaxed = true)
    private val nameService = mockk<GemNameService>(relaxed = true)
    private val repository = AddressesRepository(addressesDao, nameService)

    @Test
    fun getAddressNames_usesCacheAndFetchesOnlyMissingThenSaves() = runTest {
        val cached = ChainAddress(Chain.Ethereum, "0xcached")
        val missing = ChainAddress(Chain.Ethereum, "0xmissing")
        coEvery { addressesDao.get(Chain.Ethereum, "0xcached") } returns
            DbAddress(Chain.Ethereum, "0xcached", null, "Cached", AddressType.Contact, VerificationStatus.Verified)
        coEvery { addressesDao.get(Chain.Ethereum, "0xmissing") } returns null
        coEvery { nameService.getAddressNames(listOf(missing.toJson())) } returns
            listOf(AddressName(Chain.Ethereum, "0xmissing", "USDC", AddressType.Contact, VerificationStatus.Verified)).map { it.toJson() }

        val result = repository.getAddressNames(listOf(cached, missing))

        assertEquals(listOf("Cached", "USDC"), result.map { it.name })
        coVerify(exactly = 1) { nameService.getAddressNames(listOf(missing.toJson())) }
        coVerify { addressesDao.updateNames(any()) }
    }
}
