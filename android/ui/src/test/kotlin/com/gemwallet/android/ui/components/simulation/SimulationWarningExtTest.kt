package com.gemwallet.android.ui.components.simulation

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.ui.R
import uniffi.gemstone.SimulationSeverity
import uniffi.gemstone.SimulationWarning
import uniffi.gemstone.SimulationWarningApproval
import uniffi.gemstone.SimulationWarningType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.math.BigInteger

class SimulationWarningExtTest {
    private val assetId = mockAsset().id.toIdentifier()

    @Test
    fun finiteTokenApproval_isHidden() {
        val warning = approvalWarning(
            warning = SimulationWarningType.TokenApproval(
                SimulationWarningApproval(assetId = assetId, value = BigInteger("1")),
            ),
        )

        assertFalse(warning.isVisible())
        assertNull(warning.titleRes())
        assertNull(warning.descriptionRes())
    }

    @Test
    fun finitePermitApproval_isHidden() {
        val warning = approvalWarning(
            warning = SimulationWarningType.PermitApproval(
                SimulationWarningApproval(assetId = assetId, value = BigInteger("1")),
            ),
        )

        assertFalse(warning.isVisible())
        assertNull(warning.titleRes())
        assertNull(warning.descriptionRes())
    }

    @Test
    fun finitePermitBatchApproval_isHidden() {
        val warning = approvalWarning(
            warning = SimulationWarningType.PermitBatchApproval(BigInteger("1")),
        )

        assertFalse(warning.isVisible())
        assertNull(warning.titleRes())
        assertNull(warning.descriptionRes())
    }

    @Test
    fun unlimitedTokenApproval_usesUnlimitedWarningCopy() {
        val warning = approvalWarning(
            warning = SimulationWarningType.TokenApproval(
                SimulationWarningApproval(assetId = assetId, value = null),
            ),
        )

        assertTrue(warning.isVisible())
        assertEquals(R.string.simulation_warning_unlimited_token_approval_title, warning.titleRes())
        assertEquals(R.string.simulation_warning_unlimited_token_approval_description, warning.descriptionRes())
    }

    @Test
    fun unlimitedPermitApproval_usesUnlimitedWarningCopy() {
        val warning = approvalWarning(
            warning = SimulationWarningType.PermitApproval(
                SimulationWarningApproval(assetId = assetId, value = null),
            ),
        )

        assertTrue(warning.isVisible())
        assertEquals(R.string.simulation_warning_unlimited_token_approval_title, warning.titleRes())
        assertEquals(R.string.simulation_warning_unlimited_token_approval_description, warning.descriptionRes())
    }

    @Test
    fun unlimitedPermitBatchApproval_usesUnlimitedWarningCopy() {
        val warning = approvalWarning(
            warning = SimulationWarningType.PermitBatchApproval(null),
        )

        assertTrue(warning.isVisible())
        assertEquals(R.string.simulation_warning_unlimited_token_approval_title, warning.titleRes())
        assertEquals(R.string.simulation_warning_unlimited_token_approval_description, warning.descriptionRes())
    }

    @Test
    fun validationWarning_keepsExistingWarningBehavior() {
        val warning = SimulationWarning(
            severity = SimulationSeverity.WARNING,
            warning = SimulationWarningType.ValidationError,
            message = "Chain ID mismatch",
        )

        assertTrue(warning.isVisible())
        assertEquals(R.string.common_warning, warning.titleRes())
        assertNull(warning.descriptionRes())
    }

    @Test
    fun externallyOwnedSpender_usesSpecificWarningDescription() {
        val warning = SimulationWarning(
            severity = SimulationSeverity.WARNING,
            warning = SimulationWarningType.ExternallyOwnedSpender,
            message = null,
        )

        assertTrue(warning.isVisible())
        assertEquals(R.string.common_warning, warning.titleRes())
        assertEquals(R.string.simulation_warning_externally_owned_spender_description, warning.descriptionRes())
    }

    private fun approvalWarning(warning: SimulationWarningType): SimulationWarning {
        return SimulationWarning(
            severity = SimulationSeverity.WARNING,
            warning = warning,
            message = null,
        )
    }
}
