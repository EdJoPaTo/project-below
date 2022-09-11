# Generated by rust2rpm 21
%bcond_without check

%global crate project-below

Name:           rust-%{crate}
Version:        {{{ git_dir_version }}}
Release:        1%{?dist}
Summary:        Quickly run commands in many projects below the current directory

# Upstream license specification: MIT
License:        MIT
URL:            https://github.com/EdJoPaTo/project-below.git
VCS:            {{{ git_dir_vcs }}}

Source:         {{{ git_dir_pack }}}

ExclusiveArch:  %{rust_arches}

BuildRequires:  rust-packaging >= 21
BuildRequires:  (crate(clap/default) >= 3.0.0 with crate(clap/default) < 4.0.0~)
BuildRequires:  (crate(clap/derive) >= 3.0.0 with crate(clap/derive) < 4.0.0~)
BuildRequires:  (crate(clap/wrap_help) >= 3.0.0 with crate(clap/wrap_help) < 4.0.0~)
BuildRequires:  (crate(clap_complete/default) >= 3.0.0 with crate(clap_complete/default) < 4.0.0~)
BuildRequires:  (crate(globset/default) >= 0.4.0 with crate(globset/default) < 0.5.0~)
BuildRequires:  (crate(ignore/default) >= 0.4.0 with crate(ignore/default) < 0.5.0~)

%global _description %{expand:
Quickly run commands in many projects below the current directory.}

%description %{_description}

%package     -n %{crate}
Summary:        %{summary}

%description -n %{crate} %{_description}

%files       -n %{crate}
%license LICENSE
%doc CHANGELOG.md README.md
%{_bindir}/project-below

%prep
%autosetup -n %{crate}-%{version_no_tilde} -p1
%cargo_prep

%build
%cargo_build

%install
%cargo_install

%if %{with check}
%check
%cargo_test
%endif

%changelog
{{{ git_dir_changelog }}}
