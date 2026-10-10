// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct AcceptTermsScene: View {
    @State private var model: AcceptTermsSceneViewModel

    init(model: AcceptTermsSceneViewModel) {
        self.model = model
    }

    var body: some View {
        List {
            CalloutView(style: .header(title: model.message))
                .cleanListRow()

            ForEach(Array(model.viewState.rows.enumerated()), id: \.element.item) { index, row in
                Section {
                    Toggle(isOn: Binding(get: { row.isAccepted }, set: { _ in model.onToggle(row.item) })) {
                        Text(row.item.message)
                            .textStyle(row.isAccepted ? .body : TextStyle(font: .body, color: Colors.black.opacity(.strong)))
                    }
                    .accessibilityIdentifier("term_\(index + 1)")
                    .toggleStyle(CheckboxStyle(position: .left))
                }
            }
        }
        .safeAreaButton {
            StateButton(
                text: Localized.Onboarding.AcceptTerms.continue,
                type: .primary(model.state),
                action: model.accept,
            )
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.custom(.medium))
        .navigationTitle(model.title)
        .toolbarTitleDisplayMode(.inline)
        .toolbarInfoButton(url: model.termsAndServicesURL)
    }
}
