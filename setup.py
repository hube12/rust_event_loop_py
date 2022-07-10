from setuptools import setup, find_packages

NAME = 'rust_py'


def build_native(spec):
    # Step 1: build the rust library
    build = spec.add_external_build(
        cmd=['cargo', 'build', '--release'],
        path='.'
    )

    # Step 2: add a cffi module based on the dylib we built
    #
    # We use lambdas here for dylib and header_filename so that those are
    # only called after the external build finished.
    spec.add_cffi_module(
        module_path=NAME + '._native',
        dylib=lambda: build.find_dylib(NAME, in_path='target/release'),
        header_filename=lambda: build.find_header(NAME + '.h', in_path='target'),
        rtld_flags=['NOW', 'NODELETE']
    )


setup(
    name=NAME,
    version='1.0.0',
    packages=find_packages(),
    include_package_data=True,
    zip_safe=False,
    platforms='any',
    setup_requires=['milksnake'],
    install_requires=['cffi', 'enum34'],
    milksnake_tasks=[
        build_native,
    ]
)
