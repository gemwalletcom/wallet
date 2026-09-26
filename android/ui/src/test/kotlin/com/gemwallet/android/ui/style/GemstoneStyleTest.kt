package com.gemwallet.android.ui.style

import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.infoDescriptionRes
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemTransactionStateTone

class GemstoneStyleTest {
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
