// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

public struct ShowSecretDataScene: View {
    let model: SecretDataViewModel
    @State private var isPresentingCopyToast = false

    public init(model: SecretDataViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            Section {
                CalloutView(style: model.calloutViewStyle)
            }
            .cleanListRow()

            Section {
                SecretDataTypeView(type: model.type)
            }
            .cleanListRow()

            ListButton(
                title: Localized.Common.copy,
                image: Images.System.copy,
                action: copy,
            )
            .frame(maxWidth: .infinity, alignment: .center)
            .cleanListRow()
        }
        .safeAreaButton(isVisible: model.continueAction != nil) {
            StateButton(
                text: Localized.Common.continue,
                action: continueAction,
            )
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.custom(.medium))
        .toolbarInfoButton(url: model.docsUrl)
        .navigationTitle(model.title)
        .copyToast(
            copy: model.copy,
            isPresenting: $isPresentingCopyToast,
        )
        .detectScreenshots(docsUrl: model.docsUrl)
        .protectFromScreenRecording()
    }

    private func copy() {
        isPresentingCopyToast = true
    }

    func continueAction() {
        model.continueAction?()
    }
}
