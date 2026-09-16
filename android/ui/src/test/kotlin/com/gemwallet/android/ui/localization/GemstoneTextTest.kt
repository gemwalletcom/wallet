package com.gemwallet.android.ui.localization

import com.gemwallet.android.testkit.mockGemSimulationWarningRow
import com.gemwallet.android.ui.R
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemSimulationWarningKind
import uniffi.gemstone.SimulationSeverity

class GemstoneTextTest {

    @Test
    fun unlimitedApproval_usesUnlimitedWarningCopy() {
        val row = mockGemSimulationWarningRow(GemSimulationWarningKind.UNLIMITED_APPROVAL)

        assertEquals(R.string.simulation_warning_unlimited_token_approval_title, row.titleRes())
        assertEquals(R.string.simulation_warning_unlimited_token_approval_description, row.descriptionRes())
    }

    @Test
    fun validationWarning_keepsExistingWarningBehavior() {
        val row = mockGemSimulationWarningRow(GemSimulationWarningKind.VALIDATION_ERROR, message = "Chain ID mismatch")

        assertEquals(R.string.common_warning, row.titleRes())
        assertNull(row.descriptionRes())
    }

    @Test
    fun criticalValidationError_usesErrorCopy() {
        val row = mockGemSimulationWarningRow(GemSimulationWarningKind.VALIDATION_ERROR, severity = SimulationSeverity.CRITICAL)

        assertEquals(R.string.errors_error_occurred, row.titleRes())
        assertEquals(R.string.errors_error_occurred, row.descriptionRes())
    }

    @Test
    fun externallyOwnedSpender_usesSpecificWarningDescription() {
        val row = mockGemSimulationWarningRow(GemSimulationWarningKind.EXTERNALLY_OWNED_SPENDER)

        assertEquals(R.string.common_warning, row.titleRes())
        assertEquals(R.string.simulation_warning_externally_owned_spender_description, row.descriptionRes())
    }
}
