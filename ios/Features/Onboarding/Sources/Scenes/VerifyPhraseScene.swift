// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import PrimitivesComponents
import Style
import SwiftUI

struct VerifyPhraseScene: View {
    @State private var model: VerifyPhraseSceneViewModel
    @State private var rejectedPicks: [Int: Int] = [:]

    init(model: VerifyPhraseSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        List {
            CalloutView(style: .header(title: Localized.SecretPhrase.Confirm.QuickTest.title))
                .cleanListRow()

            Section {
                CaptureProtectedView {
                    SecretPhraseGridView(
                        rows: model.rows,
                        highlightIndex: model.wordsIndex,
                    )
                }
            }
            .cleanListRow()

            Section {
                CaptureProtectedView {
                    Grid(alignment: .center) {
                        ForEach(model.groups, id: \.self) { section in
                            GridRow(alignment: .center) {
                                ForEach(section) { row in
                                    if model.isVerified(index: row) {
                                        Button {} label: {
                                            Text(row.word)
                                        }
                                        .buttonStyle(.lightGray(paddingHorizontal: .small, paddingVertical: .tiny, glassEffect: .disabled))
                                        .disabled(true)
                                        .fixedSize()
                                        .accessibilityIdentifier("word_option")
                                    } else {
                                        Button {
                                            if !model.pickWord(index: row) {
                                                rejectedPicks[row.index, default: 0] += 1
                                            }
                                        } label: {
                                            Text(row.word)
                                        }
                                        .buttonStyle(.blueGrayPressed(paddingHorizontal: .small, paddingVertical: .tiny, glassEffect: .disabled))
                                        .fixedSize()
                                        .keyframeAnimator(initialValue: CGFloat.zero, trigger: rejectedPicks[row.index]) { content, offset in
                                            content.offset(x: offset)
                                        } keyframes: { _ in
                                            KeyframeTrack {
                                                CubicKeyframe(.small, duration: 0.05)
                                                CubicKeyframe(-.small, duration: 0.1)
                                                CubicKeyframe(.tiny, duration: 0.08)
                                                CubicKeyframe(-.tiny, duration: 0.08)
                                                CubicKeyframe(.zero, duration: 0.05)
                                            }
                                        }
                                        .accessibilityIdentifier("word_option")
                                    }
                                }
                            }
                        }
                    }
                }
                .sensoryFeedback(.error, trigger: rejectedPicks)
            }
            .cleanListRow()
        }
        .safeAreaButton {
            StateButton(
                text: Localized.Common.continue,
                type: .primary(model.buttonState),
                action: model.onContinue,
            )
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.custom(.medium))
        .navigationTitle(model.title)
        .alertSheet($model.isPresentingAlertMessage)
        .detectScreenshots(kind: .phrase)
        .protectFromScreenRecording()
    }
}
