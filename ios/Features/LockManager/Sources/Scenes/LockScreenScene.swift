// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct LockScreenScene: View {
    let model: LockSceneViewModel

    var body: some View {
        placeholderView
            .overlay(alignment: .bottom) { unlockButton }
            .animation(.smooth, value: model.isLocked)
            .frame(maxWidth: .infinity)
    }
}

// MARK: - UI Components

extension LockScreenScene {
    @ViewBuilder
    private var unlockButton: some View {
        if model.isUnlockButtonVisible {
            Button {
                model.startUnlock()
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

    private var placeholderView: some View {
        LogoView()
            .background(Colors.white)
    }
}

// MARK: - Previews

#Preview {
    LockScreenScene(model: .init())
}
