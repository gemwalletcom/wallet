// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Components
import Style
import SwiftUI

public struct SecurityScene: View {
    @State private var model: SecurityViewModel

    public init(model: SecurityViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            ForEach(Array(model.sections.enumerated()), id: \.offset) { index, section in
                Section {
                    ForEach(section.values) { row in
                        content(for: row)
                    }
                } footer: {
                    if index == 0 {
                        Text(model.authenticationFooter)
                    }
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .onChange(of: model.isEnabled, onToggleBiometrics)
        .onChange(of: model.isPrivacyLockEnabled, onToggleSecurityLock)
        .alertSheet($model.isPresentingAlertMessage)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }

    @ViewBuilder
    private func content(for row: SecurityRow) -> some View {
        switch row {
        case .authentication:
            Toggle(model.authenticationTitle, isOn: $model.isEnabled)
                .toggleStyle(AppToggleStyle())
        case .lockPeriod:
            Picker(model.lockPeriodTitle, selection: $model.lockPeriod) {
                ForEach(model.allLockPeriods) {
                    Text($0.title)
                }
            }
            .pickerStyle(.menu)
        case .privacyLock:
            Toggle(model.privacyLockTitle, isOn: $model.isPrivacyLockEnabled)
                .toggleStyle(AppToggleStyle())
        case .hideBalance:
            Toggle(model.hideBalanceTitle, isOn: $model.isHideBalanceEnabled)
                .toggleStyle(AppToggleStyle())
        }
    }
}

// MARK: - Actions

extension SecurityScene {
    private func onToggleBiometrics() {
        Task {
            await model.toggleBiometrics()
        }
    }

    private func onToggleSecurityLock() {
        model.togglePrivacyLock()
    }
}

