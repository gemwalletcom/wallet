// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct LockScreenScene: View {
    let model: LockSceneViewModel

    var body: some View {
        LogoView()
            .background(Colors.white)
            .overlay(alignment: .bottom) { unlockButton }
            .animation(.smooth, value: model.viewState.screen)
            .frame(maxWidth: .infinity)
    }
}

// MARK: - UI Components

extension LockScreenScene {
    @ViewBuilder
    private var unlockButton: some View {
        if case .lock(unlockButton: true) = model.viewState.screen {
            Button {
                model.requestUnlock()
            } label: {
                HStack {
                    if let image = model.unlockImage {
                        Image(systemName: image)
                    }
                    Text(model.unlockTitle)
                }
            }
            .buttonStyle(.blue())
            .frame(maxWidth: .scene.button.maxWidth)
            .padding()
        }
    }
}

// MARK: - Previews

#Preview {
    LockScreenScene(model: .preview)
}
