package com.gemwallet.android.features.contacts.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.data.services.store.queries.ContactsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Contact
import com.wallet.core.primitives.ContactData
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemContactServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class ContactsViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    private val contact = Contact(id = "1", name = "Alice", description = null, imageUrl = null, createdAt = 0L, updatedAt = 0L)

    private fun model(service: GemContactServiceInterface, savedStateHandle: SavedStateHandle = SavedStateHandle()): ContactsViewModel = ContactsViewModel(
        mockk<ContactsQuery> { every { this@mockk() } returns flowOf(emptyList()) },
        service,
        savedStateHandle,
        dispatcher,
        mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns
                "Error"
        },
    )

    @Test
    fun `picking a contact while adding an address saves that address on the contact`() = runTest(dispatcher) {
        val service = mockk<GemContactServiceInterface>(relaxed = true)
        val model = model(service, SavedStateHandle(mapOf(RouteArgument.Chain.key to "bitcoin", RouteArgument.Address.key to "bc1qar0")))
        var added = false

        model.addAddress(ContactData(contact, emptyList())) { added = true }
        advanceUntilIdle()

        coVerify { service.updateContact(contact.toGem(), match { it.single().address == "bc1qar0" && it.single().chain == "bitcoin" }) }
        assertTrue(added)
    }

    @Test
    fun `a deleted contact leaves no error`() = runTest(dispatcher) {
        val service = mockk<GemContactServiceInterface>(relaxed = true)
        val model = model(service)

        model.deleteContact(contact)
        advanceUntilIdle()
        Thread.sleep(50)
        advanceUntilIdle()

        assertNull(model.errorText.value)
    }

    @Test
    fun `a failed delete surfaces the error and clears on demand`() = runTest(dispatcher) {
        val service = mockk<GemContactServiceInterface>(relaxed = true) {
            coEvery { deleteContact(any()) } throws IllegalStateException("contact is in use")
        }
        val model = model(service)

        model.deleteContact(contact)
        repeat(200) {
            if (model.errorText.value != null) return@repeat
            Thread.sleep(2)
        }
        advanceUntilIdle()

        assertNotNull(model.errorText.value)

        model.clearError()
        assertNull(model.errorText.value)
    }
}
