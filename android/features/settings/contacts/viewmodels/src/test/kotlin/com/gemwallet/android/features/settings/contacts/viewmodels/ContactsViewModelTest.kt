package com.gemwallet.android.features.settings.contacts.viewmodels

import android.content.Context
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.wallet.core.primitives.Contact
import io.mockk.coEvery
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

    private fun model(service: GemContactServiceInterface): ContactsViewModel = ContactsViewModel(
        mockk<GetContacts>(relaxed = true) { every { getContacts() } returns flowOf(emptyList()) },
        service,
        dispatcher,
        mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns
                "Error"
        },
    )

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
