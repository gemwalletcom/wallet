// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Style
import Testing
import struct Gemstone.GemSimulationWarningRow

struct SimulationWarningViewModelTests {
    @Test
    func titleUsesWarningTitleWhenMessageExists() {
        let row = GemSimulationWarningRow(kind: .validationError, severity: .warning, message: "Chain ID mismatch")
        let model = SimulationWarningViewModel(row: row)

        #expect(model.id == row)
        #expect(model.title == Localized.Common.warning)
        #expect(model.message == "Chain ID mismatch")
    }

    @Test
    func titleUsesWarningTitleWhenDefaultMessageExists() {
        let model = SimulationWarningViewModel(row: GemSimulationWarningRow(kind: .unlimitedApproval, severity: .warning, message: nil))

        #expect(model.title == Localized.Simulation.Warning.UnlimitedTokenApproval.title)
        #expect(model.message == Localized.Simulation.Warning.UnlimitedTokenApproval.description)
    }

    @Test
    func colorMatchesSeverity() {
        #expect(SimulationWarningViewModel(row: GemSimulationWarningRow(kind: .validationError, severity: .critical, message: nil)).color == Colors.red)
        #expect(SimulationWarningViewModel(row: GemSimulationWarningRow(kind: .validationError, severity: .warning, message: nil)).color == Colors.orange)
    }

    @Test
    func suspiciousAddressUsesErrorOccurredTitle() {
        let model = SimulationWarningViewModel(row: GemSimulationWarningRow(kind: .suspiciousSpender, severity: .critical, message: nil))

        #expect(model.title == Localized.Errors.errorOccurred)
        #expect(model.message == Localized.Common.suspiciousAddress)
    }

    @Test
    func externallyOwnedSpenderUsesSpecificWarningDescription() {
        let model = SimulationWarningViewModel(row: GemSimulationWarningRow(kind: .externallyOwnedSpender, severity: .warning, message: nil))

        #expect(model.title == Localized.Common.warning)
        #expect(model.message == Localized.Simulation.warningExternallyOwnedSpenderDescription)
    }
}
