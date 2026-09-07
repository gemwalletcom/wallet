package com.gemwallet.android.ui.components

import com.gemwallet.android.ui.R
import com.wallet.core.primitives.TransactionState
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemTransactionStateTone

class TransactionStateExtTest {
    @Test
    fun everyStateNamesItsOwnLabel() {
        assertEquals(R.string.transaction_status_pending, TransactionState.Pending.statusLabelRes())
        assertEquals(R.string.transaction_status_pending, TransactionState.InTransit.statusLabelRes())
        assertEquals(R.string.transaction_status_confirmed, TransactionState.Confirmed.statusLabelRes())
        assertEquals(R.string.transaction_status_failed, TransactionState.Failed.statusLabelRes())
        assertEquals(R.string.transaction_status_reverted, TransactionState.Reverted.statusLabelRes())
        assertEquals(R.string.transaction_status_refunded, TransactionState.Refunded.statusLabelRes())
    }

    @Test
    fun aRefundReadsAsAnErrorAndSuccessStandsAlone() {
        assertEquals(R.drawable.transaction_state_error, GemTransactionStateTone.REFUNDED.badgeIconRes())
        assertEquals(R.string.info_transaction_error_description, GemTransactionStateTone.REFUNDED.infoDescriptionRes())
        assertEquals(R.drawable.transaction_state_success, GemTransactionStateTone.SUCCESS.badgeIconRes())
        assertEquals(R.string.info_transaction_success_description, GemTransactionStateTone.SUCCESS.infoDescriptionRes())
        assertEquals(R.drawable.transaction_state_pending, GemTransactionStateTone.PENDING.badgeIconRes())
        assertEquals(R.string.info_transaction_pending_description, GemTransactionStateTone.PENDING.infoDescriptionRes())
    }
}
