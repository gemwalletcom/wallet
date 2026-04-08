package com.gemwallet.android.ui.components

import com.gemwallet.android.ui.R
import com.wallet.core.primitives.TransactionState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TransactionStateExtTest {
    @Test
    fun inTransit_reusesPendingBadgeWithoutSpinner() {
        val state = TransactionState.InTransit

        assertEquals(R.string.transaction_status_pending, state.statusLabelRes())
        assertEquals(R.string.info_transaction_error_description, state.statusInfoDescriptionRes())
        assertEquals(R.drawable.transaction_state_pending, state.statusBadgeIconRes())
        assertEquals(TransactionStateTone.Pending, state.statusTone())
        assertTrue(state.showsStatusBadge())
        assertFalse(state.showsStatusProgress())
    }

    @Test
    fun confirmed_hidesCompactBadgeAndUsesSuccessPresentation() {
        val state = TransactionState.Confirmed

        assertEquals(R.string.transaction_status_confirmed, state.statusLabelRes())
        assertEquals(R.string.info_transaction_success_description, state.statusInfoDescriptionRes())
        assertEquals(R.drawable.transaction_state_success, state.statusBadgeIconRes())
        assertEquals(TransactionStateTone.Success, state.statusTone())
        assertFalse(state.showsStatusBadge())
        assertFalse(state.showsStatusProgress())
    }
}
