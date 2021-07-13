import hudson.tasks.test.AbstractTestResultAction
import hudson.model.Actionable
import hudson.tasks.junit.CaseResult

pipeline {
    agent none
    options {
        buildDiscarder(logRotator(numToKeepStr: '50'))
        disableConcurrentBuilds()
    }
    environment {
        GITHUB_TOKEN = credentials('githubrelease')
        AWSIP = 'ec2-18-197-145-81.eu-central-1.compute.amazonaws.com'

        TOOL_NAME = 'gtoolkit'
        MACOS_INTEL_TARGET = 'x86_64-apple-darwin'
        MACOS_M1_TARGET = 'aarch64-apple-darwin'
        WINDOWS_AMD64_TARGET = 'x86_64-pc-windows-msvc'
        LINUX_AMD64_TARGET = 'x86_64-unknown-linux-gnu'
    }

    stages {
        stage ('Parallel build') {
            parallel {
                stage ('MacOS x86_64') {
                    agent {
                        label "${MACOS_INTEL_TARGET}"
                    }
                    environment {
                        TARGET = "${MACOS_INTEL_TARGET}"
                        PATH = "$HOME/.cargo/bin:/usr/local/bin/:$PATH"
                    }

                    steps {
                        sh 'git clean -fdx'
                        sh "cargo build --bin ${TOOL_NAME} --release"

                        sh "mv target/release/${TOOL_NAME} ${TOOL_NAME}-${TARGET}"

                        stash includes: "${TOOL_NAME}-${TARGET}", name: "${TARGET}"
                    }
                }
                stage ('MacOS M1') {
                    agent {
                        label "${MACOS_M1_TARGET}"
                    }

                    environment {
                        TARGET = "${MACOS_M1_TARGET}"
                        PATH = "$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
                    }

                    steps {
                        sh 'git clean -fdx'
                        sh "cargo build --bin ${TOOL_NAME} --release"

                        sh "mv target/release/${TOOL_NAME} ${TOOL_NAME}-${TARGET}"

                        stash includes: "${TOOL_NAME}-${TARGET}", name: "${TARGET}"
                    }
                }
                stage ('Linux x86_64') {
                    agent {
                        label "${LINUX_AMD64_TARGET}"
                    }
                    environment {
                        TARGET = "${LINUX_AMD64_TARGET}"
                        PATH = "$HOME/.cargo/bin:$PATH"
                    }

                    steps {
                        sh 'git clean -fdx'
                        sh "cargo build --bin ${TOOL_NAME} --release"

                        sh "mv target/release/${TOOL_NAME} ${TOOL_NAME}-${TARGET}"

                        stash includes: "${TOOL_NAME}-${TARGET}", name: "${TARGET}"
                    }
                }
                stage ('Windows x86_64') {
                    agent {
                        label "${WINDOWS_AMD64_TARGET}"
                    }

                    environment {
                        TARGET = "${WINDOWS_AMD64_TARGET}"
                        LLVM_HOME = 'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\Llvm\\x64'
                        LIBCLANG_PATH = "${LLVM_HOME}\\bin"
                        CMAKE_PATH = 'C:\\Program Files\\CMake\\bin'
                        MSBUILD_PATH = 'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\MSBuild\\Current\\Bin'
                        CARGO_HOME = "C:\\.cargo"
                        CARGO_PATH = "${CARGO_HOME}\\bin"
                        PATH = "${CARGO_PATH};${LIBCLANG_PATH};${MSBUILD_PATH};${CMAKE_PATH};$PATH"
                    }

                    steps {
                        powershell 'git clean -fdx'

                        powershell "cargo build --bin ${TOOL_NAME} --release"
                        powershell "Move-Item -Path target/release/${TOOL_NAME}.exe -Destination ${TOOL_NAME}-${TARGET}.exe"
                        stash includes: "${TOOL_NAME}-${TARGET}.exe", name: "${TARGET}"
                    }
                }
            }
        }
        stage ('Sign and Notarize Mac') {
            agent {
                label "${MACOS_M1_TARGET}"
            }

            environment {
                TARGET = "${MACOS_M1_TARGET}"
                PATH = "$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
                CERT = credentials('devcertificate')
                APPLEPASSWORD = credentials('notarizepassword')
            }

            steps {
                sh 'git clean -fdx'
                unstash "${MACOS_INTEL_TARGET}"
                unstash "${MACOS_M1_TARGET}"
                sh "curl -o feenk-signer -LsS  https://github.com/feenkcom/feenk-signer/releases/latest/download/feenk-signer-${TARGET}"
                sh "chmod +x feenk-signer"

                sh "./feenk-signer ${TOOL_NAME}-${MACOS_INTEL_TARGET}"
                sh "./feenk-signer ${TOOL_NAME}-${MACOS_M1_TARGET}"
                sh """
                xcrun altool -t osx -f ${TOOL_NAME}-${MACOS_INTEL_TARGET} -itc_provider "77664ZXL29" --primary-bundle-id "com.feenk.gtoolkit-${MACOS_INTEL_TARGET}" --notarize-app --verbose  --username "george.ganea@feenk.com" --password "${APPLEPASSWORD}"
                xcrun altool -t osx -f ${TOOL_NAME}-${MACOS_M1_TARGET} -itc_provider "77664ZXL29" --primary-bundle-id "com.feenk.gtoolkit-${MACOS_M1_TARGET}" --notarize-app --verbose  --username "george.ganea@feenk.com" --password "${APPLEPASSWORD}"
                """
            }
        }
        stage ('Deployment') {
            agent {
                label "${LINUX_AMD64_TARGET}"
            }
            environment {
                TARGET = "${LINUX_AMD64_TARGET}"
            }
            when {
                expression {
                    (currentBuild.result == null || currentBuild.result == 'SUCCESS') && env.BRANCH_NAME.toString().equals('main')
                }
            }
            steps {
                unstash "${LINUX_AMD64_TARGET}"
                unstash "${MACOS_INTEL_TARGET}"
                unstash "${MACOS_M1_TARGET}"
                unstash "${WINDOWS_AMD64_TARGET}"

                sh "wget -O feenk-releaser https://github.com/feenkcom/releaser-rs/releases/latest/download/feenk-releaser-${TARGET}"
                sh "chmod +x feenk-releaser"

                sh """
                ./feenk-releaser \
                    --owner feenkcom \
                    --repo gtoolkit-maestro-rs \
                    --token GITHUB_TOKEN \
                    --bump-minor \
                    --auto-accept \
                    --assets \
                        ${TOOL_NAME}-${LINUX_AMD64_TARGET} \
                        ${TOOL_NAME}-${MACOS_INTEL_TARGET} \
                        ${TOOL_NAME}-${MACOS_M1_TARGET} \
                        ${TOOL_NAME}-${WINDOWS_AMD64_TARGET}.exe """
            }
        }
    }
}
