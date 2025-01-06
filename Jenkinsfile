import hudson.tasks.test.AbstractTestResultAction
import hudson.model.Actionable
import hudson.tasks.junit.CaseResult

pipeline {
    agent none
    parameters {
        choice(name: 'BUMP', choices: ['minor', 'patch', 'major'], description: 'What to bump when releasing') }
    options {
        buildDiscarder(logRotator(numToKeepStr: '50'))
        disableConcurrentBuilds()
    }
    environment {
        GITHUB_TOKEN = credentials('githubrelease')
        AWSIP = 'ec2-18-197-145-81.eu-central-1.compute.amazonaws.com'

        TOOL_NAME = 'gt-installer'
        REPOSITORY_OWNER = 'feenkcom'
        REPOSITORY_NAME = 'gtoolkit-maestro-rs'

        MACOS_INTEL_TARGET = 'x86_64-apple-darwin'
        MACOS_M1_TARGET = 'aarch64-apple-darwin'

        WINDOWS_AMD64_SERVER_NAME = 'daffy-duck'
        WINDOWS_AMD64_TARGET = 'x86_64-pc-windows-msvc'
        WINDOWS_ARM64_SERVER_NAME = 'bugs-bunny'
        WINDOWS_ARM64_TARGET = 'aarch64-pc-windows-msvc'

        LINUX_AMD64_SERVER_NAME = 'mickey-mouse'
        LINUX_AMD64_TARGET = 'x86_64-unknown-linux-gnu'
        LINUX_ARM64_SERVER_NAME = 'peter-pan'
        LINUX_ARM64_TARGET = 'aarch64-unknown-linux-gnu'
    }

    stages {
        stage ('Read tool versions') {
            agent {
                label "${MACOS_M1_TARGET}"
            }
            steps {
                script {
                    FEENK_RELEASER_VERSION = sh (
                        script: "cat feenk-releaser.version",
                        returnStdout: true
                    ).trim()
                    FEENK_SIGNER_VERSION = sh (
                        script: "cat feenk-signer.version",
                        returnStdout: true
                    ).trim()
                }
                echo "Will release using feenk-releaser ${FEENK_RELEASER_VERSION}"
                echo "Will sign using feenk-releaser ${FEENK_SIGNER_VERSION}"
            }
        }
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
                        sh "cargo build --bin ${TOOL_NAME} --release"

                        sh "mv -f target/release/${TOOL_NAME} ${TOOL_NAME}-${TARGET}"

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
                        sh "cargo build --bin ${TOOL_NAME} --release"

                        sh "mv -f target/release/${TOOL_NAME} ${TOOL_NAME}-${TARGET}"

                        stash includes: "${TOOL_NAME}-${TARGET}", name: "${TARGET}"
                    }
                }

                stage ('Linux x86_64') {
                    agent {
                        label "${LINUX_AMD64_TARGET}-${LINUX_AMD64_SERVER_NAME}"
                    }
                    environment {
                        TARGET = "${LINUX_AMD64_TARGET}"
                        PATH = "$HOME/.cargo/bin:$PATH"
                        OPENSSL_STATIC = 1
                        OPENSSL_LIB_DIR = "/usr/lib/x86_64-linux-gnu"
                        OPENSSL_INCLUDE_DIR = "/usr/include/openssl"
                    }

                    steps {
                        sh "cargo build --bin ${TOOL_NAME} --release"

                        sh "rm -rf ${TOOL_NAME}-${TARGET}"
                        sh "mv -f target/release/${TOOL_NAME} ${TOOL_NAME}-${TARGET}"

                        stash includes: "${TOOL_NAME}-${TARGET}", name: "${TARGET}"
                    }
                }
                stage ('Linux arm64') {
                    agent {
                        label "${LINUX_ARM64_TARGET}-${LINUX_ARM64_SERVER_NAME}"
                    }
                    environment {
                        TARGET = "${LINUX_ARM64_TARGET}"
                        PATH = "$HOME/.cargo/bin:$PATH"
                        OPENSSL_STATIC = 1
                        OPENSSL_LIB_DIR = "/usr/lib/aarch64-linux-gnu"
                        OPENSSL_INCLUDE_DIR = "/usr/include/openssl"
                    }

                    steps {
                        sh "cargo build --bin ${TOOL_NAME} --release"

                        sh "rm -rf ${TOOL_NAME}-${TARGET}"
                        sh "mv -f target/release/${TOOL_NAME} ${TOOL_NAME}-${TARGET}"

                        stash includes: "${TOOL_NAME}-${TARGET}", name: "${TARGET}"
                    }
                }

                stage ('Windows x86_64') {
                    agent {
                        label "${WINDOWS_AMD64_TARGET}-${WINDOWS_AMD64_SERVER_NAME}"
                    }

                    environment {
                        TARGET = "${WINDOWS_AMD64_TARGET}"
                        CARGO_HOME = "C:\\.cargo"
                        CARGO_PATH = "${CARGO_HOME}\\bin"
                        PATH = "${CARGO_PATH};$PATH"
                    }

                    steps {
                        powershell "cargo build --bin ${TOOL_NAME} --release"
                        powershell "Move-Item -Force -Path target/release/${TOOL_NAME}.exe -Destination ${TOOL_NAME}-${TARGET}.exe"
                        stash includes: "${TOOL_NAME}-${TARGET}.exe", name: "${TARGET}"
                    }
                }
                stage ('Windows arm64') {
                    agent {
                        label "${WINDOWS_ARM64_TARGET}-${WINDOWS_ARM64_SERVER_NAME}"
                    }

                    environment {
                        TARGET = "${WINDOWS_ARM64_TARGET}"
                        CARGO_HOME = "C:\\.cargo"
                        CARGO_PATH = "${CARGO_HOME}\\bin"
                        PATH = "${CARGO_PATH};$PATH"
                    }

                    steps {
                        powershell "cargo build --bin ${TOOL_NAME} --release --target ${TARGET}"
                        powershell "Move-Item -Force -Path target/${TARGET}/release/${TOOL_NAME}.exe -Destination ${TOOL_NAME}-${TARGET}.exe"
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
            }

            steps {
                unstash "${MACOS_INTEL_TARGET}"
                unstash "${MACOS_M1_TARGET}"
                sh "rm -rf feenk-signer"
                sh "curl -o feenk-signer -LsS  https://github.com/feenkcom/feenk-signer/releases/download/${FEENK_SIGNER_VERSION}/feenk-signer-${TARGET}"
                sh "chmod +x feenk-signer"

                withCredentials([file(credentialsId: 'feenk-apple-developer-certificate', variable: 'CERT')]) {
                    sh "./feenk-signer mac ${TOOL_NAME}-${MACOS_INTEL_TARGET}"
                    sh "./feenk-signer mac ${TOOL_NAME}-${MACOS_M1_TARGET}"
                }

                stash includes: "${TOOL_NAME}-${MACOS_INTEL_TARGET}", name: "${MACOS_INTEL_TARGET}"
                stash includes: "${TOOL_NAME}-${MACOS_M1_TARGET}", name: "${MACOS_M1_TARGET}"
            }
        }
        stage ('Deployment') {
            agent {
                label "${MACOS_M1_TARGET}"
            }
            environment {
                TARGET = "${MACOS_M1_TARGET}"
            }
            when {
                expression {
                    (currentBuild.result == null || currentBuild.result == 'SUCCESS') && env.BRANCH_NAME.toString().equals('main')
                }
            }
            steps {
                unstash "${LINUX_AMD64_TARGET}"
                unstash "${LINUX_ARM64_TARGET}"
                unstash "${MACOS_INTEL_TARGET}"
                unstash "${MACOS_M1_TARGET}"
                unstash "${WINDOWS_AMD64_TARGET}"
                unstash "${WINDOWS_ARM64_TARGET}"

                sh "rm -rf feenk-releaser"
                sh "curl -o feenk-releaser -LsS https://github.com/feenkcom/releaser-rs/releases/download/${FEENK_RELEASER_VERSION}/feenk-releaser-${TARGET}"
                sh "chmod +x feenk-releaser"

                sh """
                ./feenk-releaser \
                    --owner ${REPOSITORY_OWNER} \
                    --repo ${REPOSITORY_NAME} \
                    --token GITHUB_TOKEN \
                    release \
                    --bump ${params.BUMP} \
                    --auto-accept \
                    --assets \
                        ${TOOL_NAME}-${LINUX_AMD64_TARGET} \
                        ${TOOL_NAME}-${LINUX_ARM64_TARGET} \
                        ${TOOL_NAME}-${MACOS_INTEL_TARGET} \
                        ${TOOL_NAME}-${MACOS_M1_TARGET} \
                        ${TOOL_NAME}-${WINDOWS_AMD64_TARGET}.exe \
                        ${TOOL_NAME}-${WINDOWS_ARM64_TARGET}.exe \
                        scripts/installer.sh """
            }
        }
    }
}
