package com.gemwallet.android.ui.components.simulation

import com.gemwallet.android.ui.R
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemSimulationWarningKind
import uniffi.gemstone.GemSimulationWarningRow
import uniffi.gemstone.SimulationSeverity

class SimulationWarningExtTest {

    @Test
    fun unlimitedApproval_usesUnlimitedWarningCopy() {
        val row = row(GemSimulationWarningKind.UNLIMITED_APPROVAL)

        assertEquals(R.string.simulation_warning_unlimited_token_approval_title, row.titleRes())
        assertEquals(R.string.simulation_warning_unlimited_token_approval_description, row.descriptionRes())
    }

    @Test
    fun validationWarning_keepsExistingWarningBehavior() {
        val row = row(GemSimulationWarningKind.VALIDATION_ERROR, message = "Chain ID mismatch")

        assertEquals(R.string.common_warning, row.titleRes())
        assertNull(row.descriptionRes())
    }

    @Test
    fun criticalValidationError_usesErrorCopy() {
        val row = row(GemSimulationWarningKind.VALIDATION_ERROR, severity = SimulationSeverity.CRITICAL)

        assertEquals(R.string.errors_error_occurred, row.titleRes())
        assertEquals(R.string.errors_error_occurred, row.descriptionRes())
    }

    @Test
    fun externallyOwnedSpender_usesSpecificWarningDescription() {
        val row = row(GemSimulationWarningKind.EXTERNALLY_OWNED_SPENDER)

        assertEquals(R.string.common_warning, row.titleRes())
        assertEquals(R.string.simulation_warning_externally_owned_spender_description, row.descriptionRes())
    }

    private fun row(
        kind: GemSimulationWarningKind,
        severity: SimulationSeverity = SimulationSeverity.WARNING,
        message: String? = null,
    ) = GemSimulationWarningRow(kind = kind, severity = severity, message = message)
}
