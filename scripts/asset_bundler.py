import os

with open('build' + os.sep + 'bundle.h', 'w') as outfile:
    outfile.write('static const AssetEntry generated_assets[] = {\n')
    os.chdir('assets')
    for path, _, files in os.walk('.'):
        for filename in files:
            outfile.write('    {\n')
            name = path + os.sep + filename
            name = name[len(os.sep) + 1:]
            outfile.write('        "{}",\n'.format(name))
            with open(name, 'rb') as infile:
                contents = infile.read()
            outfile.write('        {},\n'.format(len(contents)))
            outfile.write('        "{}",\n'.format(''.join('\\x{:02x}'.format(byte) for byte in contents)))
            outfile.write('    },\n')
    outfile.write('    { NULL }\n')
    outfile.write('};\n')
