import Badge from '../ui/Badge';

interface ChannelCapabilitiesProps {
  capabilities: string[];
}

const ChannelCapabilities = ({ capabilities }: ChannelCapabilitiesProps) => {
  if (capabilities.length === 0) return null;
  return (
    <div className="flex flex-wrap gap-1.5 mt-2">
      {capabilities.map(cap => (
        <Badge key={cap} dot={false}>
          {cap.replace(/_/g, ' ')}
        </Badge>
      ))}
    </div>
  );
};

export default ChannelCapabilities;
