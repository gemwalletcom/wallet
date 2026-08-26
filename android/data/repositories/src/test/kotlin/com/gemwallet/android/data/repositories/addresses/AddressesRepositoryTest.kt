package com.gemwallet.android.data.repositories.addresses

import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.AddressType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.VerificationStatus
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemNameService

class AddressesRepositoryTest {

    private val addressesDao = mockk<AddressesDao>(relaxed = true)
    private val nameService = mockk<GemNameService>()
    private val repository = AddressesRepository(addressesDao, nameService)

    @Test
    fun getAddressNames_decodesCoreResponse() = runTest {
        val request = ChainAddress(Chain.Ethereum, "0xmissing")
        val name = AddressName(Chain.Ethereum, "0xmissing", "USDC", AddressType.Contact, VerificationStatus.Verified)
        coEvery { nameService.getAddressNames(listOf(request.toJson())) } returns listOf(name.toJson())

        val result = repository.getAddressNames(listOf(request))

        assertEquals(listOf(name), result)
    }
}
