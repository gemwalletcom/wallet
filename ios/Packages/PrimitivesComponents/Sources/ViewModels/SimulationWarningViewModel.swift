// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
import Style
import SwiftUI

public struct SimulationWarningViewModel: Identifiable {
    private let warning: SimulationWarning

    public init(warning: SimulationWarning) {
        self.warning = warning
    }

    public var id: SimulationWarning {
        warning
    }

    public var isVisible: Bool {
        warning.warning.isVisible
    }

    public var title: String? {
        if warning.warning == .validationError, warning.severity != .critical {
            return Localized.Common.warning
        }
        guard let warningTitle = warning.warning.warningTitle else {
            return nil
        }
        return warningDetails == nil ? nil : warningTitle
    }

    public var message: String {
        if warning.warning == .validationError, warning.severity != .critical {
            return warning.message ?? ""
        }
        return warningDetails ?? warning.warning.warningTitle ?? ""
    }

    public var color: Color {
        warning.severity.color
    }

    private var warningDetails: String? {
        warning.message ?? warning.warning.defaultMessage
    }
}

private extension SimulationWarningType {
    var isVisible: Bool {
        switch self {
        case let .tokenApproval(approval): approval.value == nil
        case let .permitApproval(approval): approval.value == nil
        case let .permitBatchApproval(value): value == nil
        case .suspiciousSpender, .externallyOwnedSpender, .nftCollectionApproval, .validationError:
            true
        }
    }

    var warningTitle: String? {
        switch self {
        case let .tokenApproval(approval): approval.value == nil ? Localized.Simulation.Warning.UnlimitedTokenApproval.title : nil
        case let .permitApproval(approval): approval.value == nil ? Localized.Simulation.Warning.UnlimitedTokenApproval.title : nil
        case let .permitBatchApproval(value): value == nil ? Localized.Simulation.Warning.UnlimitedTokenApproval.title : nil
        case .nftCollectionApproval: Localized.Simulation.Warning.NftCollectionApproval.title
        case .suspiciousSpender, .externallyOwnedSpender, .validationError: Localized.Errors.errorOccured
        }
    }

    var defaultMessage: String? {
        switch self {
        case let .tokenApproval(approval): approval.value == nil ? Localized.Simulation.Warning.UnlimitedTokenApproval.description : nil
        case let .permitApproval(approval): approval.value == nil ? Localized.Simulation.Warning.UnlimitedTokenApproval.description : nil
        case let .permitBatchApproval(value): value == nil ? Localized.Simulation.Warning.UnlimitedTokenApproval.description : nil
        case .validationError: Localized.Errors.errorOccured
        case .suspiciousSpender, .externallyOwnedSpender: Localized.Common.suspiciousAddress
        case .nftCollectionApproval: nil
        }
    }
}

private extension SimulationSeverity {
    var color: Color {
        switch self {
        case .critical:
            Colors.red
        case .low, .warning:
            Colors.orange
        }
    }
}
