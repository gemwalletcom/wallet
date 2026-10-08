// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents
import Style
import SwiftUI

struct EnableAuthenticationScene: View {
    @State private var model: EnableAuthenticationSceneViewModel

    init(model: EnableAuthenticationSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        VStack {
            Spacer()
            StateHeroView(
                systemImage: model.image,
                title: model.title,
                description: model.description,
            )
            .padding(.horizontal, .medium)
            Spacer()
        }
        .frame(maxWidth: .infinity)
        .safeAreaView {
            VStack(spacing: .medium) {
                StateButton(
                    text: model.enableTitle,
                    action: { Task { await model.enable() } },
                )
                Button(action: model.skip) {
                    Text(model.skipTitle)
                        .textStyle(TextStyle(font: .body.weight(.semibold), color: Colors.secondaryText))
                        .frame(maxWidth: .infinity, minHeight: StateButtonStyle.maxHeight)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
            .frame(maxWidth: .scene.button.maxWidth)
            .padding(.bottom, .scene.bottom)
        }
        .background(Colors.grayBackground)
        .toolbarTitleDisplayMode(.inline)
        .navigationBarBackButtonHidden()
        .interactiveDismissDisabled()
        .alertSheet($model.isPresentingAlertMessage)
    }
}
