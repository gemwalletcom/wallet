// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemInfoSheet
import Primitives
import Style
import SwiftUI

public struct InfoSheetScene: View {
    @Environment(\.dismiss) private var dismiss
    @State private var isPresentedUrl: URL? = nil

    private let model: InfoSheetModel

    public init(sheet: GemInfoSheet, onAction: InfoSheetActionHandler? = nil) {
        model = InfoSheetModel(sheet: sheet, onAction: onAction)
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
                    $0.safeAreaView {
                        actionButtons
                            .frame(maxWidth: .scene.button.maxWidth)
                            .padding(.bottom, .scene.bottom)
                    }
                }
                .safariSheet(url: $isPresentedUrl)
        }
        .presentationDetents(.forCurrentDeviceSize())
    }

    private var actionButtons: some View {
        VStack(spacing: .small) {
            if let button = model.button {
                StateButton(text: model.buttonTitle) {
                    onAction(button)
                }
            }
            ForEach(model.secondaryButtons.indices, id: \.self) { index in
                let button = model.secondaryButtons[index]
                Button {
                    onAction(button)
                } label: {
                    Text(button.title)
                        .font(.body.weight(.semibold))
                        .foregroundStyle(Colors.black)
                        .frame(maxWidth: .infinity)
                        .frame(height: StateButtonStyle.maxHeight)
                        .contentShape(Capsule())
                        .overlay {
                            Capsule().strokeBorder(Colors.gray, lineWidth: 1)
                        }
                }
                .buttonStyle(.plain)
            }
        }
    }
}

// MARK: - Actions

extension InfoSheetScene {
    private func onAction(_ button: InfoSheetButton) {
        switch button {
        case let .url(url, _):
            isPresentedUrl = url
        case let .action(_, action):
            action()
        }
    }
}
