// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct InfoSheetScene: View {
    @Environment(\.dismiss) private var dismiss
    @State private var isPresentedUrl: URL? = nil

    private let model: InfoSheetModel

    public init(type: InfoSheetType) {
        model = InfoSheetModelFactory.create(from: type)
    }

    public init(model: InfoSheetModel) {
        self.model = model
    }

    public var body: some View {
        NavigationStack {
            InfoSheetView(model: model)
                .frame(
                    maxWidth: .infinity,
                    maxHeight: .infinity,
                )
                .padding(.horizontal, .medium)
                .toolbar(content: {
                    Button("", systemImage: SystemImage.xmark) {
                        dismiss()
                    }
                    .liquidGlass { view in
                        view
                            .buttonStyle(.bordered)
                            .buttonBorderShape(.circle)
                            .padding(.top, .medium)
                    }
                })
                .if(model.shouldShowButton) {
                    $0.safeAreaButton { actionButton }
                }
                .presentationDetentsForCurrentDeviceSize()
                .safariSheet(url: $isPresentedUrl)
        }
    }

    private var actionButton: StateButton {
        StateButton(
            text: model.buttonTitle,
            action: onAction,
        )
    }
}

// MARK: - Actions

extension InfoSheetScene {
    private func onAction() {
        guard let button = model.button else { return }

        switch button {
        case let .url(url):
            isPresentedUrl = url
        case let .action(_, action):
            action()
        }
    }
}
