import os

with open('build' + os.sep + 'bundle.h', 'w') as outfile:
    outfile.write('static const AssetEntry generated_assets[] = {\n')
    for filename in os.listdir('assets'):
        outfile.write('    {\n')
        outfile.write('        "{}",\n'.format(filename))
        with open('assets' + os.sep + filename, 'rb') as infile:
            contents = infile.read()
        outfile.write('        {},\n'.format(len(contents)))
        outfile.write('        "{}",\n'.format(''.join('\\x{:02x}'.format(byte) for byte in contents)))
        outfile.write('    },\n')
    outfile.write('    { NULL }\n')
    outfile.write('};\n')
